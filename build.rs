// #[path = "src/pyo3_build_config_customize.rs"]
// mod pyo3_build_config_customize;

use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context, anyhow};
// use crate::pyo3_build_config_customize::pyo3_build_config;

/// Get the project root directory
fn root() -> PathBuf {
    match env::var("CARGO_MANIFEST_DIR") {
        Ok(s) => Some(PathBuf::from(s)),
        Err(e) => {
            println!("cargo:warning=MESSAGE: home directory not found, {:?}", e);
            None
        }
    }.unwrap()
}

/// Get build task directory
fn xtask_root() -> PathBuf {
    root().join("xtask")
}

fn write_env_file() -> Result<()> {
    let target = env::var("TARGET").unwrap();

    let target_file_path = xtask_root().join(".target.env");

    std::fs::write(&target_file_path, target).context("write env file")?;
    Ok(())
}


pub(crate) fn cargo_from_env(
    current_dir: &Path,
    args: &[&str],
    env: Option<&HashMap<String, Option<String>>>,
) -> Result<()> {
    println!("[xtask] cargo {}", &args.join(" "));
    let mut cmd = Command::new("cargo");
    let mut cmd = cmd.current_dir(current_dir);
    if let Some(env) = env {
        for item_env in env {
            if item_env.1.is_some() {
                cmd = cmd.env(item_env.0, item_env.1.as_ref().unwrap());
            } else {
                cmd = cmd.env_remove(item_env.0);
            }
        }
    }
    println!("cargo:warning=MESSAGE: build.rs cargo_from_env -- {} cargo {:?}", current_dir.display(), args);
    match cmd
        .args(args)
        .status()
        .context(format!("[xtask] cargo {}", args.join("")))?
        .success()
    {
        true => Ok(()),
        false => {
            println!("cargo:warning=MESSAGE: build.rs cargo_from_env error");
            Err(anyhow!("[xtask] command failed"))
        }
    }
}


fn run_xtask() -> Result<()> {
    let mut env = HashMap::new();
    env.insert("CARGO_MANIFEST_DIR".to_owned(), Some(xtask_root().to_string_lossy().to_string()));
    env.insert("PYO3_CONFIG_FILE".to_owned(), None);
    cargo_from_env(&xtask_root(), &["run"], Some(&env)).context("run_xtask :: cargo run")?;
    Ok(())
}


fn main() -> Result<()> {
    write_env_file()?;
    run_xtask().context("run_xtask")?;

    let pyembedded = root().join("target").join("pyembedded");
    let python_interpreter_path = pyembedded.join("python_interpreter");
    let pyo3_confg_path = pyembedded.join("pyo3-build-config-file.txt");
    let dep_confg_path = pyembedded.join("dep_cfg");
    let python_interpreter_str = read_to_string(python_interpreter_path)?;
    let dep_confg = read_to_string(dep_confg_path)?;
    println!("cargo:rustc-env=PYTHON_INTERPRETER_PATH=\"{}\"", python_interpreter_str);
    env::set_var("PYTHON_INTERPRETER_PATH",python_interpreter_str);

    println!("cargo:rustc-env=DEP_PYTHON_PYO3_CONFIG=\"{}\"", dep_confg);
    env::set_var("DEP_PYTHON_PYO3_CONFIG",dep_confg);

    // let cfg = pyo3_build_config::InterpreterConfig::from_path(pyo3_confg_path)?;

    pyo3_build_config::use_pyo3_cfgs();

    Ok(())
}