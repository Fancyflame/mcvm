use std::{env, fs, path::PathBuf};

use anyhow::{anyhow, Result};
use bootstrap::generate_module_memory;
use mas::VirtualMachine;

mod bootstrap;
mod mas;
mod parse;

fn main() -> Result<()> {
    let mut args = env::args();

    let mas_path = args
        .next()
        .ok_or_else(|| anyhow!("input file must be provided"))?;

    let mut function_dir = PathBuf::from(
        args.nth(1)
            .ok_or_else(|| anyhow!("behavior pack path must be provided"))?,
    );
    function_dir.push("functions");

    if function_dir.exists() {
        fs::remove_dir_all(&function_dir)?;
    }
    fs::create_dir(&function_dir)?;

    let size: usize = match env::var("MCVM_MEM_SIZE") {
        Ok(s) => s.parse()?,
        Err(_) => 128,
    };

    generate_module_memory(&function_dir, size)?;
    VirtualMachine::parse(&fs::read_to_string(&mas_path)?)?.generate(&function_dir)
}
