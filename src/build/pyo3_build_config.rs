use std::{env, path::Path};
use std::path::PathBuf;
use anyhow::{Result, Context, ensure, anyhow};
use pyo3_build_config::{InterpreterConfig, use_pyo3_cfgs};
use pyo3_build_config::pyo3_build_script_impl::{resolve_interpreter_config, env_var, make_cross_compile_config};



fn escape(bytes: &[u8]) -> String {
    let mut escaped = String::with_capacity(2 * bytes.len());

    for byte in bytes {
        const LUT: &[u8; 16] = b"0123456789abcdef";

        escaped.push(LUT[(byte >> 4) as usize] as char);
        escaped.push(LUT[(byte & 0x0F) as usize] as char);
    }

    escaped
}

pub(crate) fn pyo3_build_config() -> Result<()> {

    let package_path =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("get env: CARGO_MANIFEST_DIR")?);
    let python_vm_dest_path = package_path.join("target").join("pyembedded");
    // let python_vm_dest_path_str = python_vm_dest_path.to_string_lossy().to_string();
    let pyo3_config_file = python_vm_dest_path.join("pyo3-build-config-file.txt");


    let interpreter_config = InterpreterConfig::from_path(pyo3_config_file)
            .map_err(|e| anyhow!(format!("{:?}",e)))
            .context("failed to parse contents of PYO3_CONFIG_FILE")?;
    let mut buf = Vec::new();
    interpreter_config.to_writer(&mut buf).map_err(|e| anyhow!(format!("{:?}",e)))
            .context("PYO3_CONFIG_FILE to_writer")?;

    let config = escape(&buf);

    println!("cargo:rustc-env=DEP_PYTHON_PYO3_CONFIG={}", config);
    std::env::set_var("DEP_PYTHON_PYO3_CONFIG", config);
    use_pyo3_cfgs();

    make_cross_compile_config().map_err(|e| anyhow!(format!("{:?}",e)))
            .context("make_cross_compile_config")?;
    resolve_interpreter_config().map_err(|e| anyhow!(format!("{:?}",e)))
            .context("resolve_interpreter_config")?;

    Ok(())
}