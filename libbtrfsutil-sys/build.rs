extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    // try with pkg-config
    let library_result = pkg_config::probe_library("libbtrfsutil");
    if library_result.is_err() {
        // otherwise assume the default and hope for the best
        println!("cargo:rustc-link-lib=btrfsutil");
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    let include_paths = library_result.map_or(vec![], |library| library.include_paths);
    let include_args = include_paths
        .iter()
        .map(|path| format!("-I{}", path.to_string_lossy()));

    let bindings = bindgen::Builder::default()
        .clang_args(include_args)
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .size_t_is_usize(true)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
