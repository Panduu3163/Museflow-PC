fn main() {
    slint_build::compile("ui/app.slint").expect("compile MuseFlow UI");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        assert_eq!(
            std::env::var("CARGO_CFG_TARGET_ENV").as_deref(),
            Ok("gnu"),
            "This Windows resource step currently requires the documented GNU toolchain"
        );
        let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("manifest.o");
        // The pinned GCC adds its own manifest as an end file. Keep crtend, but use
        // our single explicit manifest (Common Controls v6, DPI and long paths).
        let specs = out.with_file_name("manifest.specs");
        std::fs::write(&specs, "*endfile:\ncrtend.o%s\n\n").expect("write manifest linker specs");
        println!("cargo:rustc-link-arg=-specs={}", specs.display());
        let version = std::env::var("CARGO_PKG_VERSION").unwrap();
        let numbers = [
            "CARGO_PKG_VERSION_MAJOR",
            "CARGO_PKG_VERSION_MINOR",
            "CARGO_PKG_VERSION_PATCH",
        ]
        .map(|key| std::env::var(key).unwrap().parse::<u16>().unwrap());
        let resource = out.with_file_name("application.rc");
        let manifest = std::fs::read_to_string("assets/windows/app.rc")
            .expect("read application manifest resource");
        let content = format!(
            r#"{manifest}
1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
 BLOCK "StringFileInfo"
 BEGIN
  BLOCK "040904b0"
  BEGIN
   VALUE "FileDescription", "MuseFlow PC"
   VALUE "FileVersion", "{version}"
   VALUE "ProductName", "MuseFlow PC"
   VALUE "ProductVersion", "{version}"
   VALUE "OriginalFilename", "MuseFlow-PC.exe"
  END
 END
 BLOCK "VarFileInfo"
 BEGIN
  VALUE "Translation", 0x409, 1200
 END
END
"#,
            major = numbers[0],
            minor = numbers[1],
            patch = numbers[2]
        );
        std::fs::write(&resource, content).expect("write version resource");
        let status = std::process::Command::new("windres")
            .args(["--target=pe-x86-64", "-i"])
            .arg(&resource)
            .arg("-o")
            .arg(&out)
            .status()
            .expect("run Windows resource compiler");
        assert!(status.success(), "compile application manifest");
        println!("cargo:rustc-link-arg={}", out.display());
        println!("cargo:rerun-if-changed=assets/windows/app.rc");
        println!("cargo:rerun-if-changed=assets/windows/app.manifest");
    }
}
