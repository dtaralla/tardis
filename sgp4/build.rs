/*
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::env;
use std::path::PathBuf;
use cc;
use bindgen;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/c/sgp4.c");
    println!("cargo:rerun-if-changed=src/c/SGP4.h");

    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new()
        .file("src/c/sgp4.c")
        .flag("-Wno-dangling-else")
        .compile("libsgp4.a");

    let bindings = bindgen::Builder::default()
        .header("src/c/SGP4.h")
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("c_sgp4.rs"))
        .expect("Failed to write bindings");
}
