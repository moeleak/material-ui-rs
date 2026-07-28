//! Android lifecycle, system-bar, and safe-area integration.

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::sync::Mutex;

use iced::{Color, Subscription};
use jni::objects::{JObject, JValue};
use jni::{JavaVM, jni_sig, jni_str};

/// The native Android application handle received by `android_main`.
pub use iced_winit::winit::platform::android::activity::AndroidApp;

static ANDROID_APP: Mutex<Option<AndroidApp>> = Mutex::new(None);
static SAFE_AREA: Mutex<SafeAreaInsets> = Mutex::new(SafeAreaInsets::ZERO);
static SYSTEM_BARS_STYLE: Mutex<SystemBarsStyle> = Mutex::new(SystemBarsStyle {
    edge_to_edge: true,
    light_status_icons: false,
    light_navigation_icons: false,
    status_bar_color: Color::TRANSPARENT,
    navigation_bar_color: Color::TRANSPARENT,
});

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
    if let Err(error) = refresh_system_ui() {
        eprintln!("Could not configure Android system bars: {error:?}");
    }

    if let Err(error) = launch() {
        eprintln!("Android application failed: {error:?}");
    }
}

/// Loads Android system fonts suitable for Unicode fallback.
///
/// Pass the returned fonts to [`iced::Application::font`] before starting the
/// application. Missing files are skipped because font locations differ
/// between Android vendors and releases.
#[must_use]
pub fn system_fonts() -> Vec<Cow<'static, [u8]>> {
    const PATHS: &[&str] = &[
        "/system/fonts/NotoSans-Regular.ttf",
        "/system/fonts/NotoSans-Bold.ttf",
        "/system/fonts/NotoSansDisplay-Regular.ttf",
        "/system/fonts/NotoSerif-Regular.ttf",
        "/system/fonts/NotoSansSymbols-Regular-Subsetted.ttf",
        "/system/fonts/NotoSansSymbols2-Regular.ttf",
        "/system/fonts/Roboto-Regular.ttf",
        "/system/fonts/NotoSansCJK-Regular.ttc",
        "/system/fonts/NotoSansCJK-Bold.ttc",
        "/system/fonts/NotoSansSC-Regular.otf",
        "/system/fonts/NotoSansSC-Bold.otf",
        "/system/fonts/NotoSansJP-Regular.otf",
        "/system/fonts/NotoSansJP-Bold.otf",
        "/system/fonts/NotoSansKR-Regular.otf",
        "/system/fonts/NotoSansKR-Bold.otf",
        "/product/fonts/NotoSans-Regular.ttf",
        "/product/fonts/NotoSans-Bold.ttf",
        "/product/fonts/NotoSansDisplay-Regular.ttf",
        "/product/fonts/NotoSerif-Regular.ttf",
        "/product/fonts/NotoSansSymbols-Regular-Subsetted.ttf",
        "/product/fonts/NotoSansSymbols2-Regular.ttf",
        "/product/fonts/Roboto-Regular.ttf",
        "/product/fonts/NotoSansCJK-Regular.ttc",
        "/product/fonts/NotoSansCJK-Bold.ttc",
        "/system_ext/fonts/NotoSans-Regular.ttf",
        "/system_ext/fonts/NotoSansDisplay-Regular.ttf",
        "/system_ext/fonts/NotoSerif-Regular.ttf",
        "/system_ext/fonts/NotoSansSymbols-Regular-Subsetted.ttf",
        "/system_ext/fonts/NotoSansSymbols2-Regular.ttf",
        "/system_ext/fonts/Roboto-Regular.ttf",
    ];

    PATHS
        .iter()
        .filter_map(|path| match std::fs::read(path) {
            Ok(bytes) => Some(Cow::Owned(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                eprintln!("Skipping Android system font {path}: {error}");
                None
            }
        })
        .collect()
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
        ) => refresh_system_ui().ok().map(Event::InsetsChanged),
        iced::Event::InputMethod(_) => refresh_insets().ok().map(Event::InsetsChanged),
        _ => None,
    })
}

/// Applies edge-to-edge and system-bar icon styling.
pub fn set_system_bars(style: SystemBarsStyle) -> jni::errors::Result<()> {
    if let Ok(mut current) = SYSTEM_BARS_STYLE.lock() {
        *current = style;
    }
    apply_system_ui(style)?;
    let _ = refresh_insets();
    Ok(())
}

fn refresh_system_ui() -> jni::errors::Result<SafeAreaInsets> {
    let style = SYSTEM_BARS_STYLE
        .lock()
        .map_or_else(|_| SystemBarsStyle::default(), |style| *style);
    apply_system_ui(style)?;
    refresh_insets()
}

fn apply_system_ui(style: SystemBarsStyle) -> jni::errors::Result<()> {
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
        const DRAWS_SYSTEM_BAR_BACKGROUNDS: i32 = i32::MIN;
        const TRANSLUCENT_STATUS: i32 = 0x0400_0000;
        const TRANSLUCENT_NAVIGATION: i32 = 0x0800_0000;

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
            jni_str!("addFlags"),
            jni_sig!("(I)V"),
            &[JValue::Int(DRAWS_SYSTEM_BAR_BACKGROUNDS)],
        )?;
        let _ = env.call_method(
            &window,
            jni_str!("clearFlags"),
            jni_sig!("(I)V"),
            &[JValue::Int(TRANSLUCENT_STATUS | TRANSLUCENT_NAVIGATION)],
        )?;
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

        if sdk >= 29 {
            let _ = env.call_method(
                &window,
                jni_str!("setStatusBarContrastEnforced"),
                jni_sig!("(Z)V"),
                &[JValue::Bool(false.into())],
            )?;
            let _ = env.call_method(
                &window,
                jni_str!("setNavigationBarContrastEnforced"),
                jni_sig!("(Z)V"),
                &[JValue::Bool(false.into())],
            )?;
        }

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

                let system_bars = env
                    .call_static_method(
                        jni_str!("android/view/WindowInsets$Type"),
                        jni_str!("systemBars"),
                        jni_sig!("()I"),
                        &[],
                    )?
                    .i()?;
                let _ = env.call_method(
                    &controller,
                    jni_str!("show"),
                    jni_sig!("(I)V"),
                    &[JValue::Int(system_bars)],
                )?;
            }
        }
        Ok(())
    })
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
        let density = display_density(env, activity)?;
        let (status_fallback, navigation_fallback) =
            system_bar_resource_insets(env, activity, density)?;
        if root.as_raw().is_null() {
            return Ok(SafeAreaInsets {
                status_bars: status_fallback,
                navigation_bars: navigation_fallback,
                display_cutout: Insets::ZERO,
                ime: Insets::ZERO,
            });
        }

        if sdk_int(env)? >= 30 {
            let stable = legacy_stable_insets(env, &root, density)?;
            Ok(SafeAreaInsets {
                status_bars: typed_insets(env, &root, jni_str!("statusBars"), density, true)?
                    .max(Insets {
                        top: stable.top,
                        ..Insets::ZERO
                    })
                    .max(status_fallback),
                navigation_bars: typed_insets(
                    env,
                    &root,
                    jni_str!("navigationBars"),
                    density,
                    true,
                )?
                .max(Insets {
                    left: stable.left,
                    right: stable.right,
                    bottom: stable.bottom,
                    ..Insets::ZERO
                })
                .max(navigation_fallback),
                display_cutout: typed_insets(env, &root, jni_str!("displayCutout"), density, true)?,
                ime: typed_insets(env, &root, jni_str!("ime"), density, false)?,
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

fn system_bar_resource_insets(
    env: &mut jni::Env<'_>,
    activity: &JObject<'_>,
    density: f32,
) -> jni::errors::Result<(Insets, Insets)> {
    let resources = env
        .call_method(
            activity,
            jni_str!("getResources"),
            jni_sig!("()Landroid/content/res/Resources;"),
            &[],
        )?
        .l()?;
    let dimension = |env: &mut jni::Env<'_>, name: &str| -> jni::errors::Result<f32> {
        let name = JObject::from(env.new_string(name)?);
        let kind = JObject::from(env.new_string("dimen")?);
        let package = JObject::from(env.new_string("android")?);
        let identifier = env
            .call_method(
                &resources,
                jni_str!("getIdentifier"),
                jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I"),
                &[
                    JValue::Object(&name),
                    JValue::Object(&kind),
                    JValue::Object(&package),
                ],
            )?
            .i()?;
        if identifier == 0 {
            return Ok(0.0);
        }
        Ok(env
            .call_method(
                &resources,
                jni_str!("getDimensionPixelSize"),
                jni_sig!("(I)I"),
                &[JValue::Int(identifier)],
            )?
            .i()? as f32
            / density)
    };

    Ok((
        Insets {
            top: dimension(env, "status_bar_height")?,
            ..Insets::ZERO
        },
        Insets {
            bottom: dimension(env, "navigation_bar_height")?,
            ..Insets::ZERO
        },
    ))
}

fn legacy_stable_insets(
    env: &mut jni::Env<'_>,
    root: &JObject<'_>,
    density: f32,
) -> jni::errors::Result<Insets> {
    Ok(Insets {
        left: inset_method(env, root, jni_str!("getStableInsetLeft"))? / density,
        top: inset_method(env, root, jni_str!("getStableInsetTop"))? / density,
        right: inset_method(env, root, jni_str!("getStableInsetRight"))? / density,
        bottom: inset_method(env, root, jni_str!("getStableInsetBottom"))? / density,
    })
}

fn typed_insets(
    env: &mut jni::Env<'_>,
    root: &JObject<'_>,
    kind: &jni::strings::JNIStr,
    density: f32,
    ignoring_visibility: bool,
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
            if ignoring_visibility {
                jni_str!("getInsetsIgnoringVisibility")
            } else {
                jni_str!("getInsets")
            },
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
