fn main() {
    println!("cargo:rerun-if-changed=native/bridge.m");
    println!("cargo:rerun-if-changed=native/Info.plist");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    cc::Build::new()
        .file("native/bridge.m")
        .flag("-fobjc-arc")
        .flag("-fblocks")
        .flag("-mmacosx-version-min=14.0")
        .compile("mac_native");
    for framework in [
        "Foundation",
        "AppKit",
        "CoreAudio",
        "CoreWLAN",
        "EventKit",
        "IOKit",
        "CoreGraphics",
        "Carbon",
        "Vision",
        "IOBluetooth",
        "CoreLocation",
    ] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
    let plist = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("native/Info.plist");
    println!(
        "cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,{}",
        plist.display()
    );
    println!("cargo:rustc-link-arg=-mmacosx-version-min=14.0");
}
