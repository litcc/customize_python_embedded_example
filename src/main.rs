use anyhow::{Result, Context, anyhow};
use pyembed::{MainPythonInterpreter, OxidizedPythonInterpreterConfig};
use pyembed::PythonInterpreterProfile::Python;

include!("../target/pyembedded/default_python_config.rs");


fn print_py(str: &'static str) -> Result<()> {
    let py_config: OxidizedPythonInterpreterConfig = default_python_config();
    let interp = MainPythonInterpreter::new(py_config).context("MainPythonInterpreter Init")?;

    interp
        .with_gil(|py| -> Result<()> {
            let mut code: String = r#"print(""#.to_owned();
            code.push_str(str);
            code.push_str(r#"")"#);
            py.eval(&code, None, None)?;
            Ok(())
        }).context("python 调用错")?;
    Ok(())
}

/// Third-party library calls
fn third_party_call_py() -> Result<()> {
    let py_config: OxidizedPythonInterpreterConfig = default_python_config();
    let interp = MainPythonInterpreter::new(py_config).context("MainPythonInterpreter Init")?;

    interp
        .with_gil(|py| -> Result<()> {
            //sys.modules.keys()
            // py.eval("import netaddr", None, None).map_err(|e| anyhow!("{:?}",e))?;

            let afun = py.import("sys").map_err(|e| anyhow!("{:?}",e))?
                .getattr("modules")?.getattr("keys").map_err(|e| anyhow!("{:?}",e))?;
            let list =  afun.call0().map_err(|e| anyhow!("{:?}",e))?;


            let afun2 = py.import("netaddr").map_err(|e| anyhow!("{:?}",e)).context("import netaddr");

            println!("{:?}",afun2);
            println!("{:?}",list);
            Ok(())
        }).context("python call error")?;
    Ok(())
}


fn main() -> Result<()> {
    print_py("hello1")?;
    print_py("hello2")?;
    print_py("hello3")?;

    third_party_call_py()?;
    Ok(())
}