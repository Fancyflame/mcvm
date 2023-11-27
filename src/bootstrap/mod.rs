use std::{borrow::Cow, fs, path::Path};

use anyhow::Result;
use const_format::formatcp;

pub use bin_search::gen_bin_search;

mod bin_search;

pub const PREFIX: &str = "MCVM_Memory";
pub const MEM_POINTER: &str = formatcp!("{PREFIX}_Pointer");
pub const MEM_OFFSET: &str = formatcp!("{PREFIX}_Offset");
pub const PROGRAM_COUNTER: &str = formatcp!("{PREFIX}_Pc");
pub const REG_R0: &str = formatcp!("{PREFIX}_Reg0");
pub const REG_R1: &str = formatcp!("{PREFIX}_Reg1");
pub const REG_R2: &str = formatcp!("{PREFIX}_Reg2");
pub const REG_R3: &str = formatcp!("{PREFIX}_Reg3");
pub const FUNC_LOAD: &str = formatcp!("{PREFIX}_Load");
pub const FUNC_STORE: &str = formatcp!("{PREFIX}_Store");
pub const FUNC_SWAP: &str = formatcp!("{PREFIX}_Swap");
pub const FUNC_EXEC: &str = formatcp!("{PREFIX}_Exec");

pub fn generate_module_memory(function_dir: &Path, size: usize) -> Result<()> {
    gen_bin_search(function_dir, FUNC_LOAD, MEM_POINTER, size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {REG_R0} = {PREFIX} {}",
            nth_mem_name(nth)
        )
    })?;

    gen_bin_search(function_dir, FUNC_STORE, MEM_POINTER, size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {} = {PREFIX} {REG_R0}",
            nth_mem_name(nth)
        )
    })?;

    gen_bin_search(function_dir, FUNC_SWAP, MEM_POINTER, size, |nth| {
        format!(
            "scoreboard players operation {PREFIX} {} >< {PREFIX} {REG_R0}",
            nth_mem_name(nth)
        )
    })?;

    init_memory(function_dir, "init", size)?;

    Ok(())
}

fn nth_mem_name(nth: usize) -> String {
    format!("{PREFIX}_Mem{nth}")
}

fn init_memory(func_path: &Path, cmd_name: &str, size: usize) -> std::io::Result<()> {
    // clear untracked scoreboards
    let mut content = format!("scoreboard players reset {PREFIX}\n");

    for name in (0..size).map(|name| Cow::Owned(nth_mem_name(name))).chain(
        [
            MEM_POINTER,
            MEM_OFFSET,
            PROGRAM_COUNTER,
            REG_R0,
            REG_R1,
            REG_R2,
            REG_R3,
        ]
        .into_iter()
        .map(Cow::Borrowed),
    ) {
        content += &format!(
            "scoreboard objectives add {0} dummy\n\
            scoreboard players set {PREFIX} {0} 0\n",
            name
        );
    }

    fs::write(func_path.join(format!("{cmd_name}.mcfunction")), content)
}
