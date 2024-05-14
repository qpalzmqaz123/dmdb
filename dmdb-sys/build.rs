fn main() {
    // Compile port functions
    cc::Build::new().file("dm/port.c").compile("dmdpi-port");

    // Determine target architecture
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let base_dir = if target_arch == "x86_64" {
        "dm/x86"
    } else if target_arch == "aarch64" {
        "dm/arm"
    } else if target_arch == "mips64" {
        "dm/mips"
    } else if target_arch == "loongarch64" {
        "dm/loongarch"
    } else {
        panic!("Unsupported architecture: '{target_arch}'");
    };

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .clang_arg("-DDM64")
        .header(format!("{base_dir}/include/DPI.h"))
        .header(format!("{base_dir}/include/DPIext.h"))
        .header(format!("{base_dir}/include/DPItypes.h"))
        .generate()
        .unwrap();
    bindings.write_to_file("src/bindings.rs").unwrap();

    // Link dpi
    #[cfg(feature = "bundled")]
    {
        println!("cargo:rustc-link-lib=static=dmdpi");
        println!(
            "cargo:rustc-link-search={}/{base_dir}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    #[cfg(not(feature = "bundled"))]
    println!("cargo:rustc-link-lib=dylib=dmdpi");
}
