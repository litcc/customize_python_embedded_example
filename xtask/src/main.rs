mod python_embedding_customize;

use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::ops::Deref;
use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};

const NAME: &str = "xtask";
const HELP_TEXT: &str = "\
> release
> build
> prepare
> scan
> bloat
> ci";

fn main() -> Result<()> {
    match env::args().nth(1).as_ref().map(|arg| arg.as_str()) {
        Some("release") => release().context("[xtask] main release"),
        Some("run") => run().context("[xtask] main run"),
        Some("build") => build().context("[xtask] main build"),
        Some("prepare") => prepare().context("[xtask] main prepare"),
        Some("scan") => scan().context("[xtask] main scan"),
        Some(_) => help(true).context("[xtask] main show help"),
        None => help(false).context("[xtask] main show help"),
    }
}

fn help(invalid: bool) -> Result<()> {
    match invalid {
        true => {
            eprintln!("{}", HELP_TEXT);
            Err(anyhow!("[xtask] invalid xtask provided"))
        }
        false => {
            println!("{}", HELP_TEXT);
            Ok(())
        }
    }
}

fn release() -> Result<()> {
    prepare().context("prepare")?;
    scan().context("scan")?;
    cargo(&["build", "--release"]).context("build --release")?;
    // println!("[xtask] binary size: {}", size()?);
    Ok(())
}

fn build() -> Result<()> {
    prepare()?;
    cargo(&["build"])
}

fn run() -> Result<()> {
    let mut args = env::args().enumerate()
        .filter(|e| e.0 != 0 && e.0 != 1).map(|i| i.1).collect::<Vec<_>>();
    args.insert(0, "run".to_owned());

    let args_ref = args.iter().map(String::as_str).collect::<Vec<_>>();

    prepare()?;

    let mut env = HashMap::new();
    let path = root()?
        .join("target")
        .join("pyembedded")
        .join("pyo3-build-config-file.txt")
        .to_string_lossy()
        .to_string();
    env.insert("PYO3_CONFIG_FILE".to_owned(), Some(path));

    cargo_from_env(args_ref.as_ref(), Some(&env))
}

fn prepare() -> Result<()> {
    python_embedding_customize::build_customize().context("python_embedding_customizebuild_customize")?;
    cargo(&["update"]).context("update")?;
    cargo(&["fix", "--edition-idioms", "--allow-dirty", "--allow-staged"]).context("fix")?;
    cargo(&["clippy", "--all-features", "--all-targets"]).context("clippy")

}

fn scan() -> Result<()> {
    // cargo(&["+nightly", "udeps"])?;
    cargo(&["audit"])
}

///
/// ???????????? cargo
pub(crate) fn cargo(args: &[&str]) -> Result<()> {
    cargo_from_env(args, None)
}

///
/// ?????????????????????????????? cargo
pub(crate) fn cargo_from_env(
    args: &[&str],
    env: Option<&HashMap<String, Option<String>>>,
) -> Result<()> {
    println!("[xtask] cargo {}", &args.join(" "));
    let mut cmd = Command::new("cargo");
    let mut cmd = cmd.current_dir(root()?);
    if let Some(env) = env {
        for item_env in env {
            if item_env.1.is_some() {
                cmd = cmd.env(item_env.0, item_env.1.as_ref().unwrap());
            } else {
                cmd = cmd.env_remove(item_env.0);
            }
        }
    }
    match cmd
        .args(args)
        .status()
        .context(format!("[xtask] cargo {}", args.join("")))?
        .success()
    {
        true => Ok(()),
        false => Err(anyhow!("[xtask] command failed")),
    }
}

pub(crate) fn root() -> Result<PathBuf> {
    match Path::new(&env!("CARGO_MANIFEST_DIR")).ancestors().nth(1) {
        Some(s) => Ok(s.to_path_buf()),
        None => Err(anyhow!("[xtask] could not determine repository root")),
    }
        .context("get root path")
}

// fn size() -> Result<u64> {
//     Ok(
//         File::open(root()?.join("target").join("release").join(NAME))?
//             .metadata()?
//             .len(),
//     )
// }
