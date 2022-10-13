mod tool;


use anyhow::Result;
use tool::{third_party_call_py,print_py};
fn main() -> Result<()> {

    print_py("hello1")?;
    print_py("hello2")?;
    print_py("hello3")?;

    third_party_call_py()?;
    Ok(())
}