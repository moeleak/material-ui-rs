#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    {{crate_name}}::run_android(app);
}
