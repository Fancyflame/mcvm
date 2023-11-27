use std::{fs, path::Path};

use anyhow::Result;

use super::PREFIX;

pub fn gen_bin_search<F>(
    func_path: &Path,
    cmd_name: &str,
    pointer_reg: &str,
    size: usize,
    generate: F,
) -> Result<()>
where
    F: Fn(usize) -> String,
{
    let generate = |nth| {
        let s = generate(nth);
        assert!(s.find("\n").is_none());
        s
    };

    clear_dir(func_path, cmd_name)?;

    for nth in 0..size {
        bin_search(func_path, cmd_name, pointer_reg, nth, &generate)?;
    }

    let err_msg = "say mcvm fatal error: pointer out of range";

    let entry = if size == 0 {
        err_msg.to_string()
    } else {
        let entry_fn = if size == 1 {
            generate(0)
        } else {
            format!("function {}", bin_search_fn_name(&cmd_name, size >> 1))
        };

        let upper_bound = size - 1;
        format!(
            "execute if score {PREFIX} {pointer_reg} matches {size}.. run {err_msg}\n\
            execute if score {PREFIX} {pointer_reg} matches ..{upper_bound} run {entry_fn}"
        )
    };

    fs::write(func_path.join(format!("{}.mcfunction", cmd_name)), entry)?;
    Ok(())
}

fn clear_dir(func_path: &Path, cmd_name: &str) -> Result<()> {
    let dir = func_path.join(cmd_name);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(fs::create_dir(&dir)?)
}

fn bin_search_fn_name(id: &str, nth: usize) -> String {
    format!("{id}/SearchPoint_N{nth}")
}

fn bin_search<F>(
    func_path: &Path,
    id: &str,
    pointer_reg: &str,
    nth: usize,
    generate: F,
) -> std::io::Result<()>
where
    F: Fn(usize) -> String,
{
    let zeros = nth.trailing_zeros();

    if nth == 0 {
        return Ok(());
    }

    let content = if zeros == 0 {
        // nth: xxxx1

        // xxxx0
        let lower = nth & usize::MAX << 1;

        format!(
            "execute if score {PREFIX} {pointer_reg} matches {nth} run {}\n\
            execute if score {PREFIX} {pointer_reg} matches {lower} run {}",
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
            execute if score {PREFIX} {pointer_reg} matches {nth}.. run function {}\n\
            execute if score {PREFIX} {pointer_reg} matches ..{upper_bound} run function {}\
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
