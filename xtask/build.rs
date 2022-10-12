use std::env;
use std::path::PathBuf;

fn main() {
    let target = std::env::var("TARGET").unwrap();

    let target_file_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(".target.env");

    std::fs::write(&target_file_path, target).unwrap();
}