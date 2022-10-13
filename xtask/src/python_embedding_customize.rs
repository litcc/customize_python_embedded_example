use std::collections::hash_map::RandomState;
use anyhow::Result;
use anyhow::{anyhow, Context};
use pyoxidizerlib::environment::{canonicalize_path, default_target_triple, Environment};
use pyoxidizerlib::py_packaging::distribution::{BinaryLibpythonLinkMode, DistributionCache, DistributionFlavor, PythonDistribution, PythonDistributionLocation};
use pyoxidizerlib::python_distributions::PYTHON_DISTRIBUTIONS;
use python_packaging::filesystem_scanning::find_python_resources;
use python_packaging::interpreter::{MemoryAllocatorBackend, PythonInterpreterProfile};
use python_packaging::resource::PythonResource;
use std::collections::HashMap;
use std::fs;
use std::fs::{remove_dir_all, remove_file};
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::process::Command;
use pyoxidizerlib::py_packaging::binary::LibpythonLinkMode;
use pyoxidizerlib::py_packaging::standalone_distribution::StandaloneDistribution;
use tugger_file_manifest::{FileManifest, File};
use crate::root;


/// Used to generate custom embedded parser resources
#[allow(dead_code)]
pub fn build_customize() -> Result<()> {
    // std::env::vars().for_each(|m|{
    //    println!("{}: {}",m.0,m.1);
    // });
    let package_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("get env: CARGO_MANIFEST_DIR")?);
    let root_package_path = root()?;
    let target = {
        fs::read_to_string(&package_path.join(".target.env")).context("read target env")?
    };

    let python_vm_dest_path = root_package_path.join("target").join("pyembedded");
    // let python_vm_dest_path_str = python_vm_dest_path.to_string_lossy().to_string();
    let pyo3_config_file = python_vm_dest_path.join("pyo3-build-config-file.txt");
    let pyo3_config_file_str = pyo3_config_file.to_string_lossy().to_string();
    let target_file = python_vm_dest_path.join("target");

    if python_vm_dest_path.exists() {
        remove_dir_all(&python_vm_dest_path).unwrap();
    }
    /*if target_file.exists() {
        println!("cargo:warning=MESSAGE: target_file exists");
        let target_body = std::fs::read_to_string(target_file);
        if let Ok(target_tag) = target_body {
            if target_tag != target {
                remove_dir_all(&python_vm_dest_path).unwrap();
            }
        }
    }*/

    if !(pyo3_config_file.exists() && pyo3_config_file.is_file()) {
        println!("cargo:warning=MESSAGE: The environment is missing! The embedded Python environment is being generated...");

        let mut env = Environment::new().context("Environment new")?;
        // Use system-installed rust
        env.unmanage_rust().unwrap();

        let flavor = DistributionFlavor::Standalone.to_string();
        let build_task = generate_python_embedding_artifacts(
            &root_package_path,
            &env,
            &target,
            &flavor,
            None,
            &python_vm_dest_path,
        );

        if build_task.is_ok() {
            println!("cargo:warning=MESSAGE: Python dependency environment deployed successfully");
            std::fs::write(python_vm_dest_path.join("target"), target).unwrap();
        } else {
            remove_dir_all(python_vm_dest_path).unwrap();
            let err = build_task.err().unwrap();
            return Err(err.context("Python dependency environment deployment failure"));
        }
    }
    println!("cargo:rustc-env=PYO3_CONFIG_FILE={}", pyo3_config_file_str);
    std::env::set_var("PYO3_CONFIG_FILE", pyo3_config_file_str);
    Ok(())
}


///
/// Overridden custom function
pub fn generate_python_embedding_artifacts(
    package_path: &Path,
    env: &Environment,
    target_triple: &str,
    flavor: &str,
    python_version: Option<&str>,
    dest_path: &Path,
) -> Result<()> {
    let flavor = DistributionFlavor::try_from(flavor)
        .map_err(|e| anyhow!("{}", e))
        .context("DistributionFlavor::try_from")?;
    if !dest_path.exists() {
        std::fs::create_dir_all(dest_path)
            .with_context(|| format!("creating directory {}", dest_path.display()))?;
    }

    let dest_path = canonicalize_path(dest_path)
        .context("canonicalizing destination directory")
        .context("canonicalize_path")?;

    let distribution_record = PYTHON_DISTRIBUTIONS
        .find_distribution(target_triple, &flavor, python_version)
        .ok_or_else(|| anyhow!("could not find Python distribution matching requirements"))
        .context("find_distribution")?;

    let distribution_cache = DistributionCache::new(Some(&env.python_distributions_dir()));


    let dist = distribution_cache
        .resolve_distribution(
            &distribution_record.location,
            Some(&package_path.join("target").join("python_tmp")),
        )
        .context("resolving Python distribution")
        .context("resolve_distribution")?;


    pip_install_customize(
        &env,
        dist.clone_trait().as_ref(),
        None,
        true,
        &["netaddr".to_owned()],
        &HashMap::new(),
    ).context("pip install netaddr")?;


    let host_dist = StandaloneDistribution::from_directory(&dist.base_dir)
        .context("load Python distribution")?;

    let policy = host_dist
        .create_packaging_policy()
        .context("creating packaging policy")?;

    let mut interpreter_config = host_dist
        .create_python_interpreter_config()
        .context("creating Python interpreter config")?;

    interpreter_config.config.profile = PythonInterpreterProfile::Python;
    interpreter_config.config.interactive = Some(false);
    interpreter_config.allocator_backend = MemoryAllocatorBackend::Default;


    let mut builder = host_dist.as_python_executable_builder(
        default_target_triple(),
        target_triple,
        "python",
        BinaryLibpythonLinkMode::Default,
        &policy,
        &interpreter_config,
        Some(host_dist.clone_trait()),
    )?;

    builder.set_tcl_files_path(Some("tcl".to_string()));

    builder
        .add_distribution_resources(/*None,*/ Some(Box::new(|_, b, _| {
            // println!(
            //     "distribution resources {:?} | {}",
            //     get_python_resource_type(b),
            //     b.full_name()
            // );
            Ok(())
        })))
        .context("adding distribution resources")?;

    println!("resources number {:?}", builder.iter_resources().count());

    let embedded_context = builder
        .to_embedded_python_context(env, "1")
        .context("resolving embedded context")?;

    embedded_context
        .write_files(&dest_path)
        .context("writing embedded artifact files")?;

    embedded_context
        .extra_files
        .materialize_files(&dest_path)
        .context("writing extra files")?;


    let mut m = FileManifest::default();

    for resource in find_python_resources(
        &host_dist.stdlib_path,
        host_dist.cache_tag(),
        &host_dist.python_module_suffixes()?,
        true,
        false,
    )? {
        if let PythonResource::File(file) = resource? {
            m.add_file_entry(file.path(), file.entry())?;
        } else {
            panic!("find_python_resources() should only emit File variant");
        }
    }

    m.materialize_files_with_replace(dest_path.join("stdlib"))
        .context("writing standard library")?;

    Ok(())
}


///
/// Direct installation using pip
pub fn pip_install_customize<'a, S: BuildHasher>(
    env: &Environment,
    dist: &dyn PythonDistribution,
    libpython_link_mode: Option<LibpythonLinkMode>,
    verbose: bool,
    install_args: &[String],
    extra_envs: &HashMap<String, String, S>,
) -> Result<()> {
    let temp_dir = env.temporary_directory("pyoxidizer-setup-py-install")?;

    dist.ensure_pip()?;
    let binding = PathBuf::from(dist.python_exe_path());
    let asdf = binding.parent().unwrap();
    let mut env: HashMap<String, String, RandomState> = std::env::vars().collect();
    for (k, v) in dist.resolve_distutils(libpython_link_mode.unwrap_or(LibpythonLinkMode::Static), temp_dir.path(), &[])? {
        env.insert(k, v);
    }

    for (key, value) in extra_envs.iter() {
        env.insert(key.clone(), value.clone());
    }

    // warn!("pip installing to {}", target_dir.display());

    let mut pip_args: Vec<String> = vec![
        "-m".to_string(),
        "pip".to_string(),
        "--disable-pip-version-check".to_string(),
    ];

    if verbose {
        pip_args.push("--verbose".to_string());
    }

    pip_args.extend(vec!["install".to_string()]);

    pip_args.extend(install_args.iter().cloned());

    let mut binding = Command::new(dist.python_exe_path());
    let mut command = binding.args(pip_args);

    // let mut command = Command::new(dist.python_exe_path()).args(pip_args);

    if extra_envs.contains_key("CURRENT_DIR") {
        let create_path = extra_envs
            .get("CURRENT_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("CURRENT_DIR Error"))?;
        println!("CURRENT_DIR: {:?}", create_path);
        command = command.current_dir(create_path);
    }

    let command = command.output();

    // log_command_output(&command);
    temp_dir.close().context("closing temporary directory")?;

    let output = command?;
    if !output.status.success() {
        let errlog = String::from_utf8_lossy(&output.stderr);
        println!("pip install err:\n{}", errlog);
        return Err(anyhow!("error running pip"));
    }

    let stdlog = String::from_utf8_lossy(&output.stdout);
    println!("pip install log:\n{}", stdlog);

    Ok(())
}

/// Debugging helper functions
pub(crate) fn get_python_resource_type(data: &PythonResource) -> String {
    match data {
        PythonResource::ModuleSource(_) => "ModuleSource".to_owned(),
        PythonResource::ModuleBytecodeRequest(_) => "ModuleBytecodeRequest".to_owned(),
        PythonResource::ModuleBytecode(_) => "ModuleBytecode".to_owned(),
        PythonResource::PackageResource(_) => "PackageResource".to_owned(),
        PythonResource::PackageDistributionResource(_) => "PackageDistributionResource".to_owned(),
        PythonResource::ExtensionModule(_) => "ExtensionModule".to_owned(),
        PythonResource::EggFile(_) => "EggFile".to_owned(),
        PythonResource::PathExtension(_) => "PathExtension".to_owned(),
        PythonResource::File(_) => "File".to_owned(),
    }
}
