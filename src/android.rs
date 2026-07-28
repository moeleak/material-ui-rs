//! Android lifecycle, system-bar, and safe-area integration.

#![allow(unsafe_code)]

use std::sync::Mutex;

use iced::{Color, Subscription};
use jni::objects::{JObject, JValue};
use jni::{JavaVM, jni_sig, jni_str};

/// The native Android application handle received by `android_main`.
pub use iced_winit::winit::platform::android::activity::AndroidApp;

static ANDROID_APP: Mutex<Option<AndroidApp>> = Mutex::new(None);
static SAFE_AREA: Mutex<SafeAreaInsets> = Mutex::new(SafeAreaInsets::ZERO);

/// Insets on each edge, expressed in iced logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Insets {
    /// Left inset.
    pub left: f32,
    /// Top inset.
    pub top: f32,
    /// Right inset.
    pub right: f32,
    /// Bottom inset.
    pub bottom: f32,
}

impl Insets {
    /// No inset on any edge.
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// Combines two sources by taking the largest value on every edge.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

/// Independently tracked Android window inset sources.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SafeAreaInsets {
    /// Insets occupied by the status bar.
    pub status_bars: Insets,
    /// Insets occupied by gesture or button navigation.
    pub navigation_bars: Insets,
    /// Insets occupied by a display cutout.
    pub display_cutout: Insets,
    /// Insets occupied by the on-screen keyboard.
    pub ime: Insets,
}

impl SafeAreaInsets {
    /// An empty safe area.
    pub const ZERO: Self = Self {
        status_bars: Insets::ZERO,
        navigation_bars: Insets::ZERO,
        display_cutout: Insets::ZERO,
        ime: Insets::ZERO,
    };

    /// Insets suitable for persistent page padding.
    #[must_use]
    pub fn system(self) -> Insets {
        self.status_bars
            .max(self.navigation_bars)
            .max(self.display_cutout)
    }

    /// Insets suitable for content that must stay visible above the keyboard.
    #[must_use]
    pub fn content(self) -> Insets {
        self.system().max(self.ime)
    }
}

/// System-bar presentation synchronized with an application theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemBarsStyle {
    /// Draw application content behind system bars.
    pub edge_to_edge: bool,
    /// Use dark status-bar icons on a light background.
    pub light_status_icons: bool,
    /// Use dark navigation-bar icons on a light background.
    pub light_navigation_icons: bool,
    /// Status-bar fallback color.
    pub status_bar_color: Color,
    /// Navigation-bar fallback color.
    pub navigation_bar_color: Color,
}

impl Default for SystemBarsStyle {
    fn default() -> Self {
        Self {
            edge_to_edge: true,
            light_status_icons: false,
            light_navigation_icons: false,
            status_bar_color: Color::TRANSPARENT,
            navigation_bar_color: Color::TRANSPARENT,
        }
    }
}

/// Android runtime changes observable by an iced application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// System, cutout, or IME insets changed.
    InsetsChanged(SafeAreaInsets),
}

/// Starts a shared iced application from `android_main`.
pub fn run(app: AndroidApp, launch: impl FnOnce() -> iced::Result) {
    if let Ok(mut current) = ANDROID_APP.lock() {
        *current = Some(app.clone());
    }
    iced_winit::set_android_app(app);
    if let Err(error) = set_system_bars(SystemBarsStyle::default()) {
        eprintln!("Could not configure Android system bars: {error:?}");
    }

    if let Err(error) = launch() {
        eprintln!("Android application failed: {error:?}");
    }
}

/// Returns the most recently observed safe-area insets.
#[must_use]
pub fn safe_area_insets() -> SafeAreaInsets {
    SAFE_AREA
        .lock()
        .map_or(SafeAreaInsets::ZERO, |insets| *insets)
}

/// Listens for window changes that can affect Android insets.
#[must_use]
pub fn events() -> Subscription<Event> {
    iced::event::listen_with(|event, _status, _window| match event {
        iced::Event::Window(
            iced::window::Event::Opened { .. }
            | iced::window::Event::Resized(_)
            | iced::window::Event::Rescaled(_)
            | iced::window::Event::Focused,
        )
        | iced::Event::InputMethod(_) => refresh_insets().ok().map(Event::InsetsChanged),
        _ => None,
    })
}

/// Applies edge-to-edge and system-bar icon styling.
pub fn set_system_bars(style: SystemBarsStyle) -> jni::errors::Result<()> {
    let Some(app) = android_app() else {
        return Ok(());
    };

    with_activity(&app, |env, activity| {
        const LAYOUT_STABLE: i32 = 0x0000_0100;
        const LAYOUT_HIDE_NAVIGATION: i32 = 0x0000_0200;
        const LAYOUT_FULLSCREEN: i32 = 0x0000_0400;
        const LIGHT_NAVIGATION_BAR: i32 = 0x0000_0010;
        const LIGHT_STATUS_BAR: i32 = 0x0000_2000;
        const APPEARANCE_LIGHT_STATUS_BARS: i32 = 0x0000_0008;
        const APPEARANCE_LIGHT_NAVIGATION_BARS: i32 = 0x0000_0010;
        const SOFT_INPUT_ADJUST_RESIZE: i32 = 0x0000_0010;

        let window = env
            .call_method(
                activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
        let sdk = sdk_int(env)?;

        let _ = env.call_method(
            &window,
            jni_str!("setSoftInputMode"),
            jni_sig!("(I)V"),
            &[JValue::Int(SOFT_INPUT_ADJUST_RESIZE)],
        )?;
        let _ = env.call_method(
            &window,
            jni_str!("setStatusBarColor"),
            jni_sig!("(I)V"),
            &[JValue::Int(android_color(style.status_bar_color))],
        )?;
        let _ = env.call_method(
            &window,
            jni_str!("setNavigationBarColor"),
            jni_sig!("(I)V"),
            &[JValue::Int(android_color(style.navigation_bar_color))],
        )?;

        if sdk >= 30 {
            let _ = env.call_method(
                &window,
                jni_str!("setDecorFitsSystemWindows"),
                jni_sig!("(Z)V"),
                &[JValue::Bool((!style.edge_to_edge).into())],
            )?;
        }

        let decor = env
            .call_method(
                &window,
                jni_str!("getDecorView"),
                jni_sig!("()Landroid/view/View;"),
                &[],
            )?
            .l()?;
        let current = env
            .call_method(
                &decor,
                jni_str!("getSystemUiVisibility"),
                jni_sig!("()I"),
                &[],
            )?
            .i()?;
        let layout = LAYOUT_STABLE | LAYOUT_HIDE_NAVIGATION | LAYOUT_FULLSCREEN;
        let mut visibility = if style.edge_to_edge {
            current | layout
        } else {
            current & !layout
        };
        visibility = set_flag(visibility, LIGHT_STATUS_BAR, style.light_status_icons);
        visibility = set_flag(
            visibility,
            LIGHT_NAVIGATION_BAR,
            style.light_navigation_icons,
        );
        let _ = env.call_method(
            &decor,
            jni_str!("setSystemUiVisibility"),
            jni_sig!("(I)V"),
            &[JValue::Int(visibility)],
        )?;

        if sdk >= 30 {
            let controller = env
                .call_method(
                    &window,
                    jni_str!("getInsetsController"),
                    jni_sig!("()Landroid/view/WindowInsetsController;"),
                    &[],
                )?
                .l()?;
            if !controller.as_raw().is_null() {
                let mask = APPEARANCE_LIGHT_STATUS_BARS | APPEARANCE_LIGHT_NAVIGATION_BARS;
                let mut appearance = 0;
                appearance = set_flag(
                    appearance,
                    APPEARANCE_LIGHT_STATUS_BARS,
                    style.light_status_icons,
                );
                appearance = set_flag(
                    appearance,
                    APPEARANCE_LIGHT_NAVIGATION_BARS,
                    style.light_navigation_icons,
                );
                let _ = env.call_method(
                    &controller,
                    jni_str!("setSystemBarsAppearance"),
                    jni_sig!("(II)V"),
                    &[JValue::Int(appearance), JValue::Int(mask)],
                )?;
            }
        }
        Ok(())
    })?;

    let _ = refresh_insets();
    Ok(())
}

fn refresh_insets() -> jni::errors::Result<SafeAreaInsets> {
    let Some(app) = android_app() else {
        return Ok(SafeAreaInsets::ZERO);
    };
    let insets = query_insets(&app)?;
    if let Ok(mut current) = SAFE_AREA.lock() {
        *current = insets;
    }
    Ok(insets)
}

fn android_app() -> Option<AndroidApp> {
    ANDROID_APP.lock().ok()?.clone()
}

fn query_insets(app: &AndroidApp) -> jni::errors::Result<SafeAreaInsets> {
    with_activity(app, |env, activity| {
        let window = env
            .call_method(
                activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
        let decor = env
            .call_method(
                &window,
                jni_str!("getDecorView"),
                jni_sig!("()Landroid/view/View;"),
                &[],
            )?
            .l()?;
        let root = env
            .call_method(
                &decor,
                jni_str!("getRootWindowInsets"),
                jni_sig!("()Landroid/view/WindowInsets;"),
                &[],
            )?
            .l()?;
        if root.as_raw().is_null() {
            return Ok(safe_area_insets());
        }

        let density = display_density(env, activity)?;
        if sdk_int(env)? >= 30 {
            Ok(SafeAreaInsets {
                status_bars: typed_insets(env, &root, jni_str!("statusBars"), density)?,
                navigation_bars: typed_insets(env, &root, jni_str!("navigationBars"), density)?,
                display_cutout: typed_insets(env, &root, jni_str!("displayCutout"), density)?,
                ime: typed_insets(env, &root, jni_str!("ime"), density)?,
            })
        } else {
            let system = Insets {
                left: inset_method(env, &root, jni_str!("getSystemWindowInsetLeft"))? / density,
                top: inset_method(env, &root, jni_str!("getSystemWindowInsetTop"))? / density,
                right: inset_method(env, &root, jni_str!("getSystemWindowInsetRight"))? / density,
                bottom: inset_method(env, &root, jni_str!("getSystemWindowInsetBottom"))? / density,
            };
            Ok(SafeAreaInsets {
                status_bars: Insets {
                    top: system.top,
                    ..Insets::ZERO
                },
                navigation_bars: Insets {
                    left: system.left,
                    right: system.right,
                    bottom: system.bottom,
                    ..Insets::ZERO
                },
                display_cutout: Insets::ZERO,
                ime: Insets::ZERO,
            })
        }
    })
}

fn typed_insets(
    env: &mut jni::Env<'_>,
    root: &JObject<'_>,
    kind: &jni::strings::JNIStr,
    density: f32,
) -> jni::errors::Result<Insets> {
    let mask = env
        .call_static_method(
            jni_str!("android/view/WindowInsets$Type"),
            kind,
            jni_sig!("()I"),
            &[],
        )?
        .i()?;
    let insets = env
        .call_method(
            root,
            jni_str!("getInsets"),
            jni_sig!("(I)Landroid/graphics/Insets;"),
            &[JValue::Int(mask)],
        )?
        .l()?;
    Ok(Insets {
        left: env
            .get_field(&insets, jni_str!("left"), jni_sig!("I"))?
            .i()? as f32
            / density,
        top: env
            .get_field(&insets, jni_str!("top"), jni_sig!("I"))?
            .i()? as f32
            / density,
        right: env
            .get_field(&insets, jni_str!("right"), jni_sig!("I"))?
            .i()? as f32
            / density,
        bottom: env
            .get_field(&insets, jni_str!("bottom"), jni_sig!("I"))?
            .i()? as f32
            / density,
    })
}

fn inset_method(
    env: &mut jni::Env<'_>,
    root: &JObject<'_>,
    method: &jni::strings::JNIStr,
) -> jni::errors::Result<f32> {
    Ok(env.call_method(root, method, jni_sig!("()I"), &[])?.i()? as f32)
}

fn display_density(env: &mut jni::Env<'_>, activity: &JObject<'_>) -> jni::errors::Result<f32> {
    let resources = env
        .call_method(
            activity,
            jni_str!("getResources"),
            jni_sig!("()Landroid/content/res/Resources;"),
            &[],
        )?
        .l()?;
    let metrics = env
        .call_method(
            &resources,
            jni_str!("getDisplayMetrics"),
            jni_sig!("()Landroid/util/DisplayMetrics;"),
            &[],
        )?
        .l()?;
    Ok(env
        .get_field(&metrics, jni_str!("density"), jni_sig!("F"))?
        .f()?
        .max(1.0))
}

fn sdk_int(env: &mut jni::Env<'_>) -> jni::errors::Result<i32> {
    env.get_static_field(
        jni_str!("android/os/Build$VERSION"),
        jni_str!("SDK_INT"),
        jni_sig!("I"),
    )?
    .i()
}

fn with_activity<T>(
    app: &AndroidApp,
    operation: impl FnOnce(&mut jni::Env<'_>, &JObject<'_>) -> jni::errors::Result<T>,
) -> jni::errors::Result<T> {
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    let activity_raw = app.activity_as_ptr() as jni::sys::jobject;
    vm.attach_current_thread(|env| {
        let activity = unsafe { env.as_cast_raw::<JObject>(&activity_raw)? };
        operation(env, &activity)
    })
}

const fn set_flag(value: i32, flag: i32, enabled: bool) -> i32 {
    if enabled { value | flag } else { value & !flag }
}

fn android_color(color: Color) -> i32 {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as i32;
    (channel(color.a) << 24) | (channel(color.r) << 16) | (channel(color.g) << 8) | channel(color.b)
}
