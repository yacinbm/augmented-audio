fn main() {
    uniffi_build::generate_scaffolding("./src/augmented.udl").unwrap();

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(format!("{}/src/Generated/bindings.h", crate_dir));
}