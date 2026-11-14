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
    let files = std::fs::read_to_string(root.join("assets/reading/image-files.txt")).unwrap();
    let mut code = String::from("[\n");
    let mut missing = Vec::new();
    for name in files.lines().filter(|name| !name.is_empty()) {
        let path = root.join("assets/reading").join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        if path.exists() {
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
