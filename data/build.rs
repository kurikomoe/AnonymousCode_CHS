fn main() {
    // Force rebuild
    println!("cargo:rerun-if-changed=*");

    cxx_build::bridge("src/lib.rs")
        .cpp(true)
        .flag_if_supported("/std:c++20")
        .include(format!("{}/cxx", env!("CARGO_MANIFEST_DIR")))
        .file("cxx/utils/log.cpp")
        .compile("kdata");
}
