use std::{
    env, fs,
    iter::once,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

mod bin_search;

const PREFIX: &str = "MCVM_Memory";

fn main() -> Result<()> {
    let mut args = env::args();

    let mut function_dir = PathBuf::from(
        args.nth(1)
            .ok_or_else(|| anyhow!("behavior pack path must be provided"))?,
    );
    function_dir.push("functions");

    if !function_dir.exists() {
        fs::create_dir(&function_dir)?;
    }

    let size: u32 = match env::var("MCVM_MEM_SIZE") {
        Ok(s) => s.parse()?,
        Err(_) => 128,
    };

    bin_search::gen_bin_search(&function_dir, "load", size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {PREFIX}_Reg = {PREFIX} {}",
            nth_mem_name(nth)
        )
    })?;

    bin_search::gen_bin_search(&function_dir, "store", size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {} = {PREFIX} {PREFIX}_Reg",
            nth_mem_name(nth)
        )
    })?;

    bin_search::gen_bin_search(&function_dir, "swap", size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {} >< {PREFIX} {PREFIX}_Reg",
            nth_mem_name(nth)
        )
    })?;

    init_memory(&function_dir, "init", size)?;

    Ok(())
}

fn nth_mem_name(nth: u32) -> String {
    format!("{PREFIX}_Mem{nth}")
}

fn init_memory(func_path: &Path, cmd_name: &str, size: u32) -> std::io::Result<()> {
    let mut content = String::new();

    for name in (0..size)
        .map(nth_mem_name)
        .chain(once(format!("{PREFIX}_Reg")))
        .chain(once(format!("{PREFIX}_Ptr")))
    {
        content += &format!(
            "scoreboard objectives add {0} dummy\n\
            scoreboard players set {PREFIX} {0} 0\n",
            name
        );
    }

    fs::write(func_path.join(format!("{cmd_name}.mcfunction")), content)
}
