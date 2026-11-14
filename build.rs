fn main() {
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=assets/reading/image-files.txt");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
            root.join("app.manifest").display()
        );
    }
    println!("cargo:rerun-if-changed=assets/app.ico");
    println!("cargo:rerun-if-changed=assets/app.rc");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let sdk = std::path::PathBuf::from(
            std::env::var("ProgramFiles(x86)").expect("Windows SDK location"),
        )
        .join("Windows Kits/10/bin");
        let mut compilers: Vec<_> = std::fs::read_dir(sdk)
            .expect("Windows SDK is required")
            .filter_map(Result::ok)
            .map(|entry| entry.path().join("x64/rc.exe"))
            .filter(|path| path.exists())
            .collect();
        compilers.sort();
        let compiler = compilers
            .last()
            .expect("Windows SDK resource compiler rc.exe is required");
        let resource = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("app.res");
        let status = std::process::Command::new(compiler)
            .current_dir(root.join("assets"))
            .arg("/nologo")
            .arg("/fo")
            .arg(&resource)
            .arg(root.join("assets/app.rc"))
            .status()
            .expect("Starting the resource compiler");
        assert!(status.success(), "Compiling the application icon failed");
        println!("cargo:rustc-link-arg={}", resource.display());
    }
    let files = std::fs::read_to_string(root.join("assets/reading/image-files.txt")).unwrap();
    let mut code = String::from("[\n");
    let mut missing = Vec::new();
    for name in files.lines().filter(|name| !name.is_empty()) {
        let path = root.join("assets/reading").join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        if path.exists() {
            use std::io::Read;
            let mut header = [0u8; 24];
            std::fs::File::open(&path)
                .and_then(|mut file| file.read_exact(&mut header))
                .expect("Reading illustration header");
            assert_eq!(
                &header[..8],
                b"\x89PNG\r\n\x1a\n",
                "Illustration is not a PNG: {}. Fetch Git LFS assets before building.",
                path.display()
            );
            let width = u32::from_be_bytes(header[16..20].try_into().unwrap());
            let height = u32::from_be_bytes(header[20..24].try_into().unwrap());
            assert!(
                width >= 512 && height >= 512,
                "Illustration resolution is too low: {}",
                path.display()
            );
            code.push_str(&format!("include_bytes!({:?}) as &[u8],\n", path));
        } else {
            missing.push(name.to_string());
            code.push_str("&[],\n");
        }
    }
    code.push_str("]\n");
    if std::env::var("PROFILE").as_deref() == Ok("release") && !missing.is_empty() {
        panic!("Reading illustrations are missing: {}", missing.join(", "));
    }
    std::fs::write(
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("reading_images.rs"),
        code,
    )
    .unwrap();
}
