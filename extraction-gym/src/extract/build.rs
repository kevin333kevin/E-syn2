// use std::env;
// use std::path::PathBuf;

// fn main() -> Result<(), String> {
//     let src_dir = env::var("CARGO_MANIFEST_DIR")
//         .map_err(|_| "Environmental variable `CARGO_MANIFEST_DIR` not defined.".to_string())?;

//     println!(
//         "cargo:rustc-link-search=native={}",
//         PathBuf::from(src_dir).display()
//     );

//     // Link against the static ABC library
//     println!("cargo:rerun-if-changed=./libabc.a");
//     println!("cargo:rustc-link-lib=static=abc");

//     // Link against the C++ standard library
//     println!("cargo:rustc-link-lib=dylib=stdc++");

//     // Link against additional required libraries
//     println!("cargo:rustc-link-lib=dylib=m");
//     println!("cargo:rustc-link-lib=dylib=dl");
//     println!("cargo:rustc-link-lib=dylib=readline");
//     println!("cargo:rustc-link-lib=dylib=pthread");

//     // Specify C++17 standard (uncomment if needed)
//     // println!("cargo:rustc-flags=-std=c++17");

//     Ok(())
// }