extern crate bindgen;
extern crate pkg_config;

use bindgen::callbacks::{EnumVariantValue, ItemInfo, ItemKind, ParseCallbacks};
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
struct BindgenCallbacks {
    cargo: bindgen::CargoCallbacks,
}

impl ParseCallbacks for BindgenCallbacks {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        if enum_name
            .is_some_and(|name| name.contains("btrfs_util_error") || name == "BtrfsUtilError")
        {
            original_variant_name
                .strip_prefix("BTRFS_UTIL_ERROR_")
                .or_else(|| original_variant_name.strip_prefix("BTRFS_UTIL_"))
                .map(str::to_owned)
        } else {
            None
        }
    }

    fn item_name(&self, item_info: ItemInfo) -> Option<String> {
        match item_info.kind {
            ItemKind::Type if item_info.name == "btrfs_util_error" => {
                Some("BtrfsUtilError".to_owned())
            }
            _ => None,
        }
    }

    fn header_file(&self, filename: &str) {
        self.cargo.header_file(filename);
    }

    fn include_file(&self, filename: &str) {
        self.cargo.include_file(filename);
    }

    fn read_env_var(&self, key: &str) {
        self.cargo.read_env_var(key);
    }
}

fn main() {
    // try with pkg-config, it will handle cargo output on success
    let include_paths = match pkg_config::probe_library("libbtrfsutil") {
        Ok(lib) => lib.include_paths,
        Err(_) => {
            // otherwise assume the default and hope for the best
            println!("cargo:rustc-link-lib=btrfsutil");
            vec![]
        }
    };

    println!("cargo:rerun-if-changed=wrapper.h");

    let clang_args = include_paths
        .iter()
        .map(|path| format!("-I{}", path.to_string_lossy()));

    let bindings = bindgen::Builder::default()
        .clang_args(clang_args)
        .header("wrapper.h")
        .parse_callbacks(Box::new(BindgenCallbacks {
            cargo: bindgen::CargoCallbacks::new(),
        }))
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .newtype_enum("btrfs_util_error")
        .size_t_is_usize(true)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
