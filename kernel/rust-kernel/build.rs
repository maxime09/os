use std::env;
use std::path::PathBuf;

fn main() {

    let header_path = PathBuf::from("../src/rust_export.h")
        .canonicalize()
        .expect("Cannot canonicakuze path");
    let header_path_str = header_path.to_str().expect("Path is not a valid string");

    let bindings = bindgen::Builder::default()
        .header(header_path_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .use_core()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}