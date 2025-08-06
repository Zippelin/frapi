use std::fs;

extern crate winres;

fn main() {
    let build_target = format!("builds/{}", env!("CARGO_PKG_VERSION"));

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    };

    // Stop here, need to add macos pack
    if cfg!(target_os = "macos") {
        let bin_name = std::env::var("CARGO_PKG_NAME").unwrap();

        let macos_dir = format!("{build_target}/macos");

        let app_dir = format!("{macos_dir}/{bin_name}.app");
        fs::create_dir_all(&app_dir)
            .expect(format!("Could not create output dir: {build_target}").as_str());

        fs::create_dir_all(format!("{app_dir}/Contents/MacOS")).expect("Cant cretate MacOS Dir");
        fs::create_dir_all(format!("{app_dir}/Contents/Resources"))
            .expect("Cant cretate Resources Dir");

        let info_plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>{}</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>"#,
            bin_name,
            bin_name,
            bin_name,
            env!("CARGO_PKG_VERSION")
        );

        fs::write(format!("{app_dir}/Contents/Info.plist"), info_plist)
            .expect("Cant write Info.plist");

        fs::copy(
            "assets/icon.icns",
            format!("{app_dir}/Contents/Resources/icon.icns"),
        )
        .expect("Cant copy Icon");
    };
}
