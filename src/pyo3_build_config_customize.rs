use std::{env, path::Path};
use std::error::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;
use anyhow::{Result, Context, ensure, anyhow};
use pyo3_build_config::{InterpreterConfig, use_pyo3_cfgs};
use pyo3_build_config::pyo3_build_script_impl::{resolve_interpreter_config, make_cross_compile_config};
use crate::root;


fn escape(bytes: &[u8]) -> String {
    let mut escaped = String::with_capacity(2 * bytes.len());

    for byte in bytes {
        const LUT: &[u8; 16] = b"0123456789abcdef";

        escaped.push(LUT[(byte >> 4) as usize] as char);
        escaped.push(LUT[(byte & 0x0F) as usize] as char);
    }

    escaped
}

fn configure(interpreter_config: Option<InterpreterConfig>, path: Option<&Path>, name: &str) -> Result<bool> {
    let target = if path.is_some() {
        let path_home = path.unwrap();
        if !path_home.exists() {
            create_dir_all(&path_home.parent().unwrap()).unwrap()
        }
        PathBuf::from(path_home)
    } else {
        Path::new(&env::var_os("OUT_DIR").unwrap()).join(name)
    };
    if let Some(config) = interpreter_config {
        config
            .to_writer(&mut std::fs::File::create(&target).with_context(|| {
                format!("failed to write config file at {}", target.display())
            })?).map_err(|e| anyhow!("{:?}",e)).context("config to_writer")?;
        Ok(true)
    } else {
        std::fs::File::create(&target)
            .with_context(|| format!("failed to create new file at {}", target.display()))?;
        Ok(false)
    }
}

/// If PYO3_CONFIG_FILE is set, copy it into the crate.
fn config_file(path: &Path) -> Result<InterpreterConfig> {
    if path.exists() && path.is_file() {
        let interpreter_config = InterpreterConfig::from_path(path)
            .map_err(|e| anyhow!(format!("{:?}",e)))
            .context("failed to parse contents of PYO3_CONFIG_FILE")?;
        Ok(interpreter_config)
    } else {
        Err(anyhow!("PYO3_CONFIG_FILE not found").context("config_file"))
    }
}


pub(crate) fn pyo3_build_config() -> Result<()> {
    let root_path = root();

    let python_vm_dest_path = root_path.join("target").join("pyembedded");
    let pyo3_config_file = python_vm_dest_path.join("pyo3-build-config-file.txt");

    // let path = match env::var("TARGET") {
    //     Ok(s) => {
    //         let path = env::var("OUT_DIR").map_err(|e|anyhow!("OUT_DIR not found {:?}",e))?;
    //         let mut path = PathBuf::from(path);
    //         path.push(Path::new(&s));
    //         path.push("pyo3-build-config.txt");
    //         Ok(path)
    //     },
    //     Err(e)=>{
    //         Err(anyhow!("OUT_DIR not found {:?}",e))
    //     }
    // }?;
    //
    // println!("cargo:warning=MESSAGE: pyo3-build-config path {:?}", path);
    // configure(Some(config_file()?),Some(&path), "pyo3-build-config.txt")?;
    //
    // configure(Some(config_file()?),None, "pyo3-build-config-file.txt")?;
    // configure(Some(config_file()?),None, "pyo3-build-config.txt")?;

    let mut interpreter_config = config_file(&pyo3_config_file)?;
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