#[unsafe(no_mangle)]
fn android_main(app: material_ui_rs::android::AndroidApp) {
    material_ui_rs::android::run(app, {{crate_name}}::run);
}
