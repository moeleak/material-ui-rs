//! Android lifecycle, system-bar, and safe-area integration.

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::sync::{Mutex, MutexGuard, OnceLock};

use std::time::{Duration, Instant};

use iced::{Color, Subscription};
use jni::objects::{JObject, JValue};
use jni::{JavaVM, jni_sig, jni_str};

/// The native Android application handle received by `android_main`.
pub use iced_winit::winit::platform::android::activity::AndroidApp;

mod state;
mod subscription;

pub use state::{Insets, SafeAreaInsets, SystemBarsStyle};
use state::{RefreshState, Snapshot, WakeListeners};

struct Runtime {
    app: Option<AndroidApp>,
    state: RefreshState,
    desired_style: SystemBarsStyle,
    applied_style: Option<SystemBarsStyle>,
    dirty: bool,
    last_error: Option<String>,
    listeners: WakeListeners,
    next_poll: Instant,
}

impl Runtime {
    fn wake(&mut self) {
        self.listeners.wake();
    }
}

fn runtime() -> MutexGuard<'static, Runtime> {
    static RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            Mutex::new(Runtime {
                app: None,
                state: RefreshState::new(),
                desired_style: SystemBarsStyle::default(),
                applied_style: None,
                dirty: true,
                last_error: None,
                listeners: WakeListeners::default(),
                next_poll: Instant::now(),
            })
        })
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn activity_lifecycle(active: bool) {
    let mut runtime = runtime();
    runtime.state.set_active(active);
    if active {
        runtime.applied_style = None;
        runtime.dirty = true;
    }
    runtime.wake();
    drop(runtime);
    if active {
        let _ = queue_refresh();
    }
}

fn invalidate(reapply_style: bool) {
    let mut runtime = runtime();
    runtime.dirty = true;
    if reapply_style {
        runtime.applied_style = None;
    }
    runtime.wake();
}

// At most one JNI callback is queued. The callback never waits for the native
// event loop, which can itself be inside a synchronized Activity callback.
fn queue_refresh() -> bool {
    let (app, request, style, apply_style) = {
        let mut runtime = runtime();
        if !runtime.dirty {
            return false;
        }
        let Some(app) = runtime.app.clone() else {
            return false;
        };
        let Some(request) = runtime.state.begin_request() else {
            return false;
        };
        runtime.dirty = false;
        runtime.next_poll = Instant::now() + Duration::from_millis(100);
        (
            app,
            request,
            runtime.desired_style,
            runtime.applied_style != Some(runtime.desired_style),
        )
    };
    let callback_app = app.clone();
    app.run_on_java_main_thread(Box::new(move || {
        {
            let mut guard = runtime();
            if !guard.state.is_current(request) {
                if guard.state.finish_request(request) {
                    guard.wake();
                }
                drop(guard);
                let _ = queue_refresh();
                return;
            }
        }
        let result = (|| {
            if apply_style && apply_system_ui(&callback_app, style)? {
                let mut runtime = runtime();
                if runtime.state.is_current(request) {
                    runtime.applied_style = Some(style);
                }
            }
            if callback_app.native_window().is_none() {
                return Ok(None);
            }
            query_insets(&callback_app)
        })();
        let mut guard = runtime();
        if !guard.state.finish_request(request) {
            return;
        }
        if guard.state.active && guard.state.generation == request.generation {
            match result {
                Ok(Some(snapshot)) => {
                    let _ = guard.state.accept(request.generation, snapshot);
                    guard.last_error = None;
                }
                Ok(None) => {}
                Err(error) => {
                    let message = format!("{error:?}");
                    if guard.last_error.as_ref() != Some(&message) {
                        eprintln!("Could not refresh Android system UI: {message}");
                        guard.last_error = Some(message);
                    }
                }
            }
        }
        guard.wake();
        drop(guard);
        // A newer explicit request must not depend on an events() subscriber.
        // Failures themselves do not mark dirty, so this cannot spin on errors.
        let _ = queue_refresh();
    }));
    true
}

/// Android runtime changes observable by an iced application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// System, cutout, or IME insets changed.
    InsetsChanged(SafeAreaInsets),
}

/// Starts a shared iced application from `android_main`.
pub fn run(app: AndroidApp, launch: impl FnOnce() -> iced::Result) {
    {
        let mut runtime = runtime();
        runtime.app = Some(app.clone());
        runtime.state.begin_session();
        runtime.last_error = None;
        runtime.applied_style = None;
        runtime.dirty = true;
    }
    iced_winit::winit::platform::android::set_activity_lifecycle_listener(Some(activity_lifecycle));
    iced_winit::set_android_app(app);
    if let Err(error) = launch() {
        eprintln!("Android application failed: {error:?}");
    }
    activity_lifecycle(false);
    iced_winit::winit::platform::android::set_activity_lifecycle_listener(None);
    runtime().app = None;
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
    runtime().state.snapshot.raw
}

/// Insets still overlapping the native rendering surface, in iced logical pixels.
///
/// Unlike [`safe_area_insets`], these subtract space already consumed by Android
/// window resizing. Use these for padding and the raw IME inset for visibility.
#[must_use]
pub fn layout_insets() -> SafeAreaInsets {
    runtime().state.snapshot.layout
}

/// Observes inset changes without continuously redrawing the application.
#[must_use]
pub fn events() -> Subscription<Event> {
    iced_winit::futures::subscription::from_recipe(subscription::AndroidEvents)
}

/// Queues edge-to-edge and system-bar icon styling on Android's UI thread.
///
/// `Ok(())` means the request was accepted. Asynchronous platform failures are
/// logged and retried while the activity is resumed; the last valid insets stay
/// available. Repeating an unchanged style does not reconfigure the window.
pub fn set_system_bars(style: SystemBarsStyle) -> jni::errors::Result<()> {
    {
        let mut runtime = runtime();
        if runtime.desired_style == style {
            return Ok(());
        }
        runtime.desired_style = style;
        runtime.dirty = true;
        runtime.wake();
    }
    let _ = queue_refresh();
    Ok(())
}

fn apply_system_ui(app: &AndroidApp, style: SystemBarsStyle) -> jni::errors::Result<bool> {
    with_activity(app, |env, activity| {
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

        if sdk >= 28 {
            let _ = env.call_method(
                &window,
                jni_str!("setNavigationBarDividerColor"),
                jni_sig!("(I)V"),
                &[JValue::Int(0)],
            )?;
            let attributes = env
                .call_method(
                    &window,
                    jni_str!("getAttributes"),
                    jni_sig!("()Landroid/view/WindowManager$LayoutParams;"),
                    &[],
                )?
                .l()?;
            env.set_field(
                &attributes,
                jni_str!("layoutInDisplayCutoutMode"),
                jni_sig!("I"),
                JValue::Int(if style.edge_to_edge { 1 } else { 0 }),
            )?;
            let _ = env.call_method(
                &window,
                jni_str!("setAttributes"),
                jni_sig!("(Landroid/view/WindowManager$LayoutParams;)V"),
                &[JValue::Object(&attributes)],
            )?;
        }

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
        if sdk < 30 {
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
        }

        if sdk >= 30 {
            let controller = env
                .call_method(
                    &window,
                    jni_str!("getInsetsController"),
                    jni_sig!("()Landroid/view/WindowInsetsController;"),
                    &[],
                )?
                .l()?;
            if controller.as_raw().is_null() {
                return Ok(false);
            }
            {
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
        Ok(true)
    })
}

fn query_insets(app: &AndroidApp) -> jni::errors::Result<Option<Snapshot>> {
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
        // Attachment and rotation can briefly remove the insets object. Do not
        // replace a valid snapshot with guessed resource dimensions or zeroes.
        if root.as_raw().is_null() {
            return Ok(None);
        }
        let density = display_density(env, activity)?;
        let sdk = sdk_int(env)?;
        let consumed = consumed_insets(env, activity, &decor, app, density)?;
        let typed = if sdk >= 30 {
            let result: jni::errors::Result<SafeAreaInsets> = (|| {
                Ok(SafeAreaInsets {
                    status_bars: typed_insets(env, &root, jni_str!("statusBars"), density, false)?,
                    navigation_bars: typed_insets(
                        env,
                        &root,
                        jni_str!("navigationBars"),
                        density,
                        false,
                    )?,
                    display_cutout: typed_insets(
                        env,
                        &root,
                        jni_str!("displayCutout"),
                        density,
                        true,
                    )?,
                    ime: typed_insets(env, &root, jni_str!("ime"), density, false)?,
                })
            })();
            match result {
                Ok(insets) => Some(insets),
                Err(_error) => {
                    env.exception_clear();
                    None
                }
            }
        } else {
            None
        };
        let raw = if let Some(typed) = typed {
            // Typed API values, including zero and lateral navigation, are
            // authoritative. Resource dimensions must never enlarge them.
            typed
        } else {
            let system = Insets {
                left: inset_method(env, &root, jni_str!("getSystemWindowInsetLeft"))? / density,
                top: inset_method(env, &root, jni_str!("getSystemWindowInsetTop"))? / density,
                right: inset_method(env, &root, jni_str!("getSystemWindowInsetRight"))? / density,
                bottom: (inset_method(env, &root, jni_str!("getSystemWindowInsetBottom"))?
                    / density)
                    .max(consumed.bottom),
            };
            let stable = legacy_stable_insets(env, &root, density)?;
            let cutout = if sdk >= 28 {
                let cutout = env
                    .call_method(
                        &root,
                        jni_str!("getDisplayCutout"),
                        jni_sig!("()Landroid/view/DisplayCutout;"),
                        &[],
                    )?
                    .l()?;
                if cutout.as_raw().is_null() {
                    Insets::ZERO
                } else {
                    Insets {
                        left: inset_method(env, &cutout, jni_str!("getSafeInsetLeft"))? / density,
                        top: inset_method(env, &cutout, jni_str!("getSafeInsetTop"))? / density,
                        right: inset_method(env, &cutout, jni_str!("getSafeInsetRight"))? / density,
                        bottom: inset_method(env, &cutout, jni_str!("getSafeInsetBottom"))?
                            / density,
                    }
                }
            } else {
                Insets::ZERO
            };
            state::legacy_insets(system, stable, cutout)
        };
        Ok(Some(Snapshot {
            raw,
            layout: raw.remaining(consumed),
        }))
    })
}

fn consumed_insets(
    env: &mut jni::Env<'_>,
    activity: &JObject<'_>,
    decor: &JObject<'_>,
    app: &AndroidApp,
    density: f32,
) -> jni::errors::Result<Insets> {
    let Some(native) = app.native_window() else {
        return Ok(Insets::ZERO);
    };
    // NativeActivity's rendering view fills android.R.id.content. Its origin
    // plus ANativeWindow dimensions describe the actual rendering viewport.
    let content = env
        .call_method(
            activity,
            jni_str!("findViewById"),
            jni_sig!("(I)Landroid/view/View;"),
            &[JValue::Int(0x0102_0002)],
        )?
        .l()?;
    if content.as_raw().is_null() {
        return Ok(Insets::ZERO);
    }
    let location = env.new_int_array(2)?;
    let _ = env.call_method(
        &content,
        jni_str!("getLocationInWindow"),
        jni_sig!("([I)V"),
        &[JValue::Object(location.as_ref())],
    )?;
    let mut coordinates = [0; 2];
    location.get_region(env, 0, &mut coordinates)?;
    let (width, height) = if sdk_int(env)? >= 30 {
        // WindowMetrics include space covered by the IME even when DecorView
        // has already been resized to exclude it.
        let manager = env
            .call_method(
                activity,
                jni_str!("getWindowManager"),
                jni_sig!("()Landroid/view/WindowManager;"),
                &[],
            )?
            .l()?;
        let metrics = env
            .call_method(
                &manager,
                jni_str!("getCurrentWindowMetrics"),
                jni_sig!("()Landroid/view/WindowMetrics;"),
                &[],
            )?
            .l()?;
        let bounds = env
            .call_method(
                &metrics,
                jni_str!("getBounds"),
                jni_sig!("()Landroid/graphics/Rect;"),
                &[],
            )?
            .l()?;
        (
            inset_method(env, &bounds, jni_str!("width"))?,
            inset_method(env, &bounds, jni_str!("height"))?,
        )
    } else {
        (
            inset_method(env, decor, jni_str!("getWidth"))?,
            inset_method(env, decor, jni_str!("getHeight"))?,
        )
    };
    Ok(state::consumed_edges(
        iced::Size::new(width, height),
        iced::Point::new(coordinates[0] as f32, coordinates[1] as f32),
        iced::Size::new(native.width() as f32, native.height() as f32),
        density,
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
    let density = env
        .get_field(&metrics, jni_str!("density"), jni_sig!("F"))?
        .f()?;
    Ok(if density.is_finite() && density > 0.0 {
        density
    } else {
        1.0
    })
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
        let result = operation(env, &activity);
        if result.is_err() {
            env.exception_clear();
        }
        result
    })
}

const fn set_flag(value: i32, flag: i32, enabled: bool) -> i32 {
    if enabled { value | flag } else { value & !flag }
}

fn android_color(color: Color) -> i32 {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as i32;
    (channel(color.a) << 24) | (channel(color.r) << 16) | (channel(color.g) << 8) | channel(color.b)
}
