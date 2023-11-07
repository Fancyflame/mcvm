use std::{fs, path::Path};

use anyhow::{anyhow, Result};

use crate::PREFIX;

pub fn gen_bin_search<F>(func_path: &Path, cmd_name: &str, size: u32, generate: F) -> Result<()>
where
    F: Fn(u32) -> String,
{
    let generate = |nth| {
        let s = generate(nth);
        assert!(s.find("\n").is_none());
        s
    };

    if !size.is_power_of_two() {
        return Err(anyhow!("memory size must be power of 2"));
    }

    let id = format!("{PREFIX}_bin_search_{cmd_name}");

    let dir = func_path.join(&id);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir(&dir)?;

    for nth in 0..size {
        bin_search(func_path, &id, nth, &generate)?;
    }

    let err_msg = "say mcvm fatal error: \
    out of memory, please increase your memory size at compile time";

    let entry = if size == 0 {
        err_msg.to_string()
    } else {
        let entry_fn = if size == 1 {
            generate(0)
        } else {
            bin_search_fn_name(&id, size >> 1)
        };

        let upper_bound = size - 1;
        format!(
            "execute if score {PREFIX} {PREFIX}_Ptr matches {size}.. run {err_msg}\n\
            execute if score {PREFIX} {PREFIX}_Ptr matches ..{upper_bound} run function {entry_fn}"
        )
    };

    fs::write(func_path.join(format!("{}.mcfunction", cmd_name)), entry)?;
    Ok(())
}

fn bin_search_fn_name(id: &str, nth: u32) -> String {
    format!("{id}/SearchPoint_N{nth}")
}

fn bin_search<F>(func_path: &Path, id: &str, nth: u32, generate: F) -> std::io::Result<()>
where
    F: Fn(u32) -> String,
{
    let zeros = nth.trailing_zeros();

    if nth == 0 {
        return Ok(());
    }

    let content = if zeros == 0 {
        // nth: xxxx1

        // xxxx0
        let lower = nth & u32::MAX << 1;

        format!(
            "execute if score {PREFIX} {PREFIX}_Ptr matches {nth} run function {}\n\
            execute if score {PREFIX} {PREFIX}_Ptr matches {lower} run function {}",
            generate(nth),
            generate(lower)
        )
    } else {
        // nth: xx10000

        // xx11000
        let higher = (1 << (zeros - 1)) | nth;

        // xx01000
        let lower = !(1 << zeros) & higher;

        let upper_bound = nth - 1;

        format!(
            "\
            execute if score {PREFIX} {PREFIX}_Ptr matches {nth}.. run function {}\n\
            execute if score {PREFIX} {PREFIX}_Ptr matches ..{upper_bound} run function {}\
        ",
            bin_search_fn_name(id, higher),
            bin_search_fn_name(id, lower)
        )
    };

    fs::write(
        func_path.join(format!("{}.mcfunction", bin_search_fn_name(id, nth))),
        content,
    )
}

/*fn nth_unit_fn_name(id: &str, nth: u32) -> String {
    format!("{id}/N{nth}")
}

fn nth_unit<F>(func_path: &Path, id: &str, nth: u32, generate: F) -> std::io::Result<()>
where
    F: Fn(u32) -> String,
{
    fs::write(
        func_path.join(format!("{}.mcfunction", nth_unit_fn_name(id, nth))),
        generate(nth),
    )
}*/
