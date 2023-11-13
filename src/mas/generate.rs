use std::{fs, path::Path};

use anyhow::Result;

use crate::{
    bootstrap::{
        FUNC_LOAD, FUNC_STORE, MEM_OFFSET, MEM_POINTER, PREFIX, REG_L0, REG_L1, REG_L2, REG_L3,
    },
    mas::{CalcOp, ExprCmpIn},
};

use super::{CmpOp, Instruction, Register, VirtualMachine};

impl VirtualMachine<'_> {
    pub fn generate(&self, save_as: impl AsRef<Path>) -> Result<()> {
        let mangler = mangle_label();

        for (label, instrustions) in &self.blocks {
            let mut path = save_as.as_ref().join(mangler(label));
            path.set_extension("mcfunction");

            let mut output = String::new();
            for inst in instrustions {
                translate(&mut output, &mangler, *inst)?;
            }

            fs::write(path, output)?;
        }

        Ok(())
    }
}

fn mangle_label() -> impl Fn(&str) -> String {
    let uuid = format!("{:x}", rand::random::<u64>());
    move |label: &str| {
        if label == "__main__" {
            "main".to_string()
        } else {
            format!("{PREFIX}_{label}_mangled_{uuid}")
        }
    }
}

fn register(reg: Register) -> &'static str {
    match reg {
        Register::L0 => REG_L0,
        Register::L1 => REG_L1,
        Register::L2 => REG_L2,
        Register::L3 => REG_L3,
    }
}

fn decode_string(input: &str) -> String {
    input.replace("\\", "")
}

fn translate<F>(buffer: &mut String, mangler: F, inst: Instruction) -> Result<()>
where
    F: Fn(&str) -> String,
{
    let command = match inst {
        Instruction::Branch(b) => format!("function {}", mangler(b)),

        Instruction::BranchIf(bi) => format!(
            "execute unless score {PREFIX} {REG_L0} matches 0 run function {}\n",
            mangler(bi)
        ),

        Instruction::BranchIfNot(bn) => format!(
            "execute if score {PREFIX} {REG_L0} matches 0 run function {}\n",
            mangler(bn)
        ),

        Instruction::Calculate(opr) => {
            let opr_str = match opr {
                CalcOp::Add => "+=",
                CalcOp::Sub => "-=",
                CalcOp::Mul => "*=",
                CalcOp::Div => "/=",
                CalcOp::Rem => "%=",
                CalcOp::Min => "<",
                CalcOp::Max => ">",
            };

            format!("scoreboard players operation {PREFIX} {REG_L0} {opr_str} {PREFIX} {REG_L1}\n")
        }

        Instruction::Call { offset_inc, label } => {
            let function = mangler(label);
            format!(
                "\
                scoreboard players add {PREFIX} {MEM_OFFSET} {offset_inc}\n\
                function {function}\n\
                scoreboard players add {PREFIX} {MEM_OFFSET} -{offset_inc}\n\
            "
            )
        }

        Instruction::Compare(opr) => {
            let opr_str = match opr {
                CmpOp::Equals => "=",
                CmpOp::GreaterEq => ">=",
                CmpOp::GreaterThan => ">",
                CmpOp::LessEq => "<=",
                CmpOp::LessThan => "<",
            };

            format!("\
                execute if score {PREFIX} {REG_L0} {opr_str} {PREFIX} {REG_L1} run scoreboard players set {PREFIX} {REG_L0} 1\n\
                execute unless score {PREFIX} {REG_L0} matches 1 run scoreboard players set {PREFIX} {REG_L0} 0\n\
            ")
        }

        Instruction::CompareIn(expr) => {
            let matches = match expr {
                ExprCmpIn::Value(v) => v.to_string(),
                ExprCmpIn::Range(lb, ub) => format!(
                    "{}..{}",
                    lb.as_ref().map(<_>::to_string).unwrap_or_default(),
                    ub.as_ref().map(<_>::to_string).unwrap_or_default()
                ),
            };

            format!("\
                execute if score {PREFIX} {REG_L0} matches {matches} run scoreboard players set {PREFIX} {REG_L0} 1\n\
                execute unless score {PREFIX} {REG_L0} matches 1 run scoreboard players set {PREFIX} {REG_L0} 0\n\
            ")
        }

        Instruction::Move { dst, src } => {
            format!(
                "scoreboard players operation {PREFIX} {} = {PREFIX} {} \n",
                register(dst),
                register(src)
            )
        }

        Instruction::Load { addr } => {
            format!(
                "\
                scoreboard players set {PREFIX} {MEM_POINTER} {addr}\n\
                scoreboard players operation {PREFIX} {MEM_POINTER} += {PREFIX} {MEM_OFFSET}\n\
                function {FUNC_LOAD}\n\
            "
            )
        }

        Instruction::Random { min, max } => {
            format!("scoreboard players random {PREFIX} {REG_L0} {min} {max}\n")
        }

        Instruction::RawCommand(cmd) => {
            let mut string = decode_string(cmd);
            string += "\n";
            string
        }

        Instruction::Set { dst, value } => {
            let dst = register(dst);
            format!("scoreboard players set {PREFIX} {dst} {value}\n")
        }

        Instruction::Store { addr } => {
            format!(
                "\
                scoreboard players set {PREFIX} {MEM_POINTER} {addr}\n\
                scoreboard players operation {PREFIX} {MEM_POINTER} += {PREFIX} {MEM_OFFSET}\n\
                function {FUNC_STORE}\n\
            "
            )
        }

        Instruction::Debug { line, info } => {
            format!("say (at: {line}) {}\n", decode_string(info))
        }

        Instruction::Log(msg) => {
            format!("say {}\n", decode_string(msg))
        }
    };

    *buffer += &command;
    Ok(())
}
