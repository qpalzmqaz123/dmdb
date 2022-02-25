fn main() {
    println!("cargo:rustc-link-lib=dylib=dmdpi");

    println!("cargo:rerun-if-changed=dm/include/DPI.h");
    println!("cargo:rerun-if-changed=dm/include/DPIext.h");
    println!("cargo:rerun-if-changed=dm/include/DPItypes.h");

    let bindings = bindgen::Builder::default()
        .header("dm/include/DPI.h")
        .header("dm/include/DPIext.h")
        .header("dm/include/DPItypes.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();

    bindings.write_to_file("src/bindings.rs").unwrap();
}
