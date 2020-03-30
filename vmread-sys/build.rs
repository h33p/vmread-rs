extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(debug_assertions) {
        println!("cargo:rustc-link-lib=asan");
    }
    println!("cargo:rerun-if-changed=wrapper.h");

    let src = [
        "vmread/wintools.c",
        "vmread/pmparser.c",
        "vmread/mem.c",
        if cfg!(feature="kmod_rw") || cfg!(feature="internal_rw") {
            "vmread/intmem.c"
        } else {
            "vmread/vmmem.c"
        },
    ];

    for i in &src {
        println!("cargo:rerun-if-changed={}", i);
    }

    let mut builder = cc::Build::new();

    let mut build = builder
        .files(src.iter())
        .define("LMODE",
            if cfg!(feature="internal_rw") {
                "MODE_QEMU_INJECT"
            } else {
                "MODE_EXTERNAL"
            })
        ;

    if cfg!(debug_assertions) {
        build = build.flag("-fsanitize=address")
            .define("MVERBOSE", "4")
            ;
    }

    if cfg!(feature="kmod_rw") {
        build = build.define("KMOD_MEMMAP", None);
    }

    build.compile("vmread");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_default(true)
        .whitelist_function(".*Mem.*")
        .whitelist_function("VTranslate")
        .whitelist_function(".*Context")
        .whitelist_function("GetNTHeader")
        .whitelist_function(".*Export.*")
        .whitelist_function(".*Proc.*")
        .whitelist_function(".*Module.*")
        .whitelist_function("GetPeb.*")
        .whitelist_var("vmread_dfile")
        .blacklist_type("_IO_.*")
        .blacklist_type("FILE")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
