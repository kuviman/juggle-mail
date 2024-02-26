use batbox_android as android;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android::App) {
    android::init(app);
    android::set_file_mode(android::FileMode::Assets);
    juggle_mail::run();
}
