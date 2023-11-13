use std::{fs, path::Path};

use anyhow::Result;
use const_format::formatcp;

mod bin_search;

pub const PREFIX: &str = "MCVM_Memory";
pub const MEM_POINTER: &str = formatcp!("{PREFIX}_Pointer");
pub const MEM_OFFSET: &str = formatcp!("{PREFIX}_Offset");
pub const REG_R0: &str = formatcp!("{PREFIX}_RegR0");
pub const REG_R1: &str = formatcp!("{PREFIX}_RegR1");
pub const REG_R2: &str = formatcp!("{PREFIX}_RegR2");
pub const REG_R3: &str = formatcp!("{PREFIX}_RegR3");
pub const FUNC_LOAD: &str = formatcp!("{PREFIX}_Load");
pub const FUNC_STORE: &str = formatcp!("{PREFIX}_Store");
pub const FUNC_SWAP: &str = formatcp!("{PREFIX}_Swap");

pub fn generate_module_memory(function_dir: &Path, size: u32) -> Result<()> {
    bin_search::gen_bin_search(function_dir, FUNC_LOAD, size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {REG_R0} = {PREFIX} {}",
            nth_mem_name(nth)
        )
    })?;

    bin_search::gen_bin_search(function_dir, FUNC_STORE, size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {} = {PREFIX} {REG_R0}",
            nth_mem_name(nth)
        )
    })?;

    bin_search::gen_bin_search(function_dir, FUNC_SWAP, size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {} >< {PREFIX} {REG_R0}",
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
    // clear untracked scoreboards
    let mut content = format!("scoreboard players reset {PREFIX}\n");

    for name in (0..size).map(nth_mem_name).chain(
        [MEM_POINTER, MEM_OFFSET, REG_R0, REG_R1, REG_R2, REG_R3]
            .into_iter()
            .map(<_>::to_string),
    ) {
        content += &format!(
            "scoreboard objectives add {0} dummy\n\
            scoreboard players set {PREFIX} {0} 0\n",
            name
        );
    }

    fs::write(func_path.join(format!("{cmd_name}.mcfunction")), content)
}
