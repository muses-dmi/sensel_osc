extern crate cc;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // generate bindings
    let bindings = bindgen::Builder::default()
        .header("external/sensel-api/sensel-lib/src/sensel.h")
        .header("external/sensel-api/sensel-lib/src/sensel_register_map.h")
        .rustified_enum(".*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // link to libsensel

    // link to existing installation
    // this only works if host == target
    if cfg!(target_os = "windows") && cfg!(target_arch = "x86") {
        println!(r"cargo:rustc-link-search=C:\Program Files\Sensel\SenselLib\x86");
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        println!(r"cargo:rustc-link-search=C:\Program Files\Sensel\SenselLib\x64");
    } else if (cfg!(target_os = "macos") && cfg!(target_arch = "x86_64")) ||
        (cfg!(target_os = "linux") && (
            cfg!(target_arch = "x86_64") ||
            cfg!(target_arch = "x86") ||
            cfg!(target_arch = "arm")
        ))
    {
        // LibSensel for macos (/usr/local/lib) and linux (/usr/lib) should already be in the path
    } else {
        unimplemented!("forces not available for target")
    }
    println!(r"cargo:rustc-link-lib=sensel");

}
