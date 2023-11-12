use std::{fs, iter::once, path::Path};

use anyhow::Result;
use const_format::formatcp;

mod bin_search;

const PREFIX: &str = "MCVM_Memory";

pub fn generate_module_memory(function_dir: &Path, size: u32) -> Result<()> {
    bin_search::gen_bin_search(function_dir, formatcp!("{PREFIX}_Load"), size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {PREFIX}_Reg = {PREFIX} {}",
            nth_mem_name(nth)
        )
    })?;

    bin_search::gen_bin_search(function_dir, formatcp!("{PREFIX}_Store"), size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {} = {PREFIX} {PREFIX}_Reg",
            nth_mem_name(nth)
        )
    })?;

    bin_search::gen_bin_search(function_dir, formatcp!("{PREFIX}_Swap"), size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {} >< {PREFIX} {PREFIX}_Reg",
            nth_mem_name(nth)
        )
    })?;

    init_memory(function_dir, "init", size)?;

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
