
#[path = "src/build/python_embedding_customize.rs"]
pub(crate) mod python_embedding_customize;

#[path = "src/build/pyo3_build_config.rs"]
pub(crate) mod pyo3_build_config_customize;


use std::path::PathBuf;
use anyhow::{Context, Result};

fn main() -> Result<()> {

    let package_path =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("get env: CARGO_MANIFEST_DIR")?);
    let build_path = package_path.join("target");
    println!("cargo:rerun-if-changed={}", build_path.display());

    python_embedding_customize::build_customize().map_err(|e| {
        println!("cargo:warning=MESSAGE: build_python_customize error {:?}", e);
        // println!("cargo:warning=MESSAGE: pyo3_build_config_customize error");
        e
    }).context("build_python_customize run")?;

    pyo3_build_config_customize::pyo3_build_config().map_err(|e| {
        println!("cargo:warning=MESSAGE: pyo3_build_config_customize error {:?}", e);
        // println!("cargo:warning=MESSAGE: pyo3_build_config_customize error");
        e
    }).context("pyo3_build_config_customize run")?;

    println!("cargo:warning=MESSAGE: build done!");
    Ok(())
}
