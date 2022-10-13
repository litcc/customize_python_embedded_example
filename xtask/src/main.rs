// #[path = "src/python_embedding_customize.rs"]
mod python_embedding_customize;
mod pyo3_build_config_customize;

use std::path::{Path, PathBuf};
use anyhow::{anyhow, Context, Result};


pub(crate) fn root() -> Result<PathBuf> {
    match Path::new(&env!("CARGO_MANIFEST_DIR")).ancestors().nth(1) {
        Some(s) => Ok(s.to_path_buf()),
        None => Err(anyhow!("[xtask] could not determine repository root")),
    }
        .context("get root path")
}

fn main() -> Result<()> {
    python_embedding_customize::build_customize().context("python_embedding_customizebuild_customize")?;
    pyo3_build_config_customize::pyo3_build_config().context("pyo3_build_config")?;
    Ok(())
}
