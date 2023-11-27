use std::{borrow::Cow, path::Path};

use anyhow::Result;

use crate::{
    bootstrap::{
        FUNC_EXEC, FUNC_LOAD, FUNC_STORE, MEM_OFFSET, MEM_POINTER, PREFIX, PROGRAM_COUNTER, REG_R0,
        REG_R1, REG_R2, REG_R3,
    },
    mas::{CalcOp, ExprCmpIn},
};

use self::ctx::Context;

use super::{CmpOp, Instruction, Register, VirtualMachine};

mod ctx;

impl VirtualMachine<'_> {
    pub fn generate(&self, save_as: impl AsRef<Path>) -> Result<()> {
        let mut ctx = Context::new();

        for label in self.blocks.keys() {
            ctx.insert_label(*label, if *label == "main" { false } else { true });
        }

        for (label, function) in &self.blocks {
            let mut label = Cow::Borrowed(*label);

            for inst in &function.instructions {
                if let Some(new_l) = translate(&label, &mut ctx, *inst)? {
                    label = Cow::Owned(new_l);
                }
            }
        }

        Ok(ctx.generate(save_as)?)
    }
}

fn register(reg: Register) -> &'static str {
    match reg {
        Register::R0 => REG_R0,
        Register::R1 => REG_R1,
        Register::R2 => REG_R2,
        Register::R3 => REG_R3,
    }
}

fn decode_string(input: &str) -> String {
    input.replace("\\", "")
}

// returns some means switch to a new label
fn translate<'c>(label: &str, ctx: &'c mut Context, inst: Instruction) -> Result<Option<String>> {
    let mut switch = None;

    let command = match inst {
        Instruction::Branch(b) => {
            // let code generate to an unreachable block
            switch = Some(ctx.new_anonymous_label());

            let label = ctx.get_label(b);
            let b_id = label.id();
            let b_fn = label.fn_name();

            format!(
                "scoreboard players set {PREFIX} {PROGRAM_COUNTER} {b_id}\n\
                function {b_fn}\n"
            )
        }

        Instruction::BranchIf(bi) => {
            let an_label = switch.insert(ctx.new_anonymous_label());

            let if_true_exec = ctx.get_label(bi).fn_name();
            let if_false_exec = ctx.get_label(an_label).fn_name();

            format!(
                "execute unless score {PREFIX} {REG_R0} matches 0 run function {if_true_exec}\n\
                execute if score {PREFIX} {REG_R0} matches 0 run function {if_false_exec}\n",
            )
        }

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

            format!("scoreboard players operation {PREFIX} {REG_R0} {opr_str} {PREFIX} {REG_R1}\n")
        }

        Instruction::Call {
            mut offset_inc,
            label,
        } => {
            let ret_label = ctx.new_anonymous_label();
            let ret_block = ctx.get_label(switch.insert(ret_label));
            let this_id = ctx.get_label(label).id();
            let ret_pc = offset_inc;
            offset_inc += 1;

            ret_block.push_str(format!(
                "scoreboard players add {PREFIX} {MEM_OFFSET} -{offset_inc}\n"
            ));

            let function = ctx.get_label(label).fn_name();
            format!(
                "scoreboard players set {PREFIX} {REG_R0} {this_id}\n\
                scoreboard players set {PREFIX} {MEM_POINTER} {ret_pc}\n\
                function {FUNC_STORE}\n\
                scoreboard players add {PREFIX} {MEM_OFFSET} {offset_inc}\n\
                function {function}\n",
            )
        }

        Instruction::Compare(opr) => {
            let mut if_ = "if";

            let opr_str = match opr {
                CmpOp::Equals => "=",
                CmpOp::NotEquals => {
                    if_ = "unless";
                    "="
                }
                CmpOp::GreaterEq => ">=",
                CmpOp::GreaterThan => ">",
                CmpOp::LessEq => "<=",
                CmpOp::LessThan => "<",
            };

            format!("\
                execute {if_} score {PREFIX} {REG_R0} {opr_str} {PREFIX} {REG_R1} run scoreboard players set {PREFIX} {REG_R0} 1\n\
                execute unless score {PREFIX} {REG_R0} matches 1 run scoreboard players set {PREFIX} {REG_R0} 0\n\
            ")
        }

        Instruction::CompareIn { not, opr: expr } => {
            let if_ = if not { "unless" } else { "if" };

            let matches = match expr {
                ExprCmpIn::Value(v) => v.to_string(),
                ExprCmpIn::Range(lb, ub) => format!(
                    "{}..{}",
                    lb.as_ref().map(<_>::to_string).unwrap_or_default(),
                    ub.as_ref().map(<_>::to_string).unwrap_or_default()
                ),
            };

            format!("\
                execute {if_} score {PREFIX} {REG_R0} matches {matches} run scoreboard players set {PREFIX} {REG_R0} 1\n\
                execute unless score {PREFIX} {REG_R0} matches 1 run scoreboard players set {PREFIX} {REG_R0} 0\n\
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
                "scoreboard players set {PREFIX} {MEM_POINTER} {addr}\n\
                scoreboard players operation {PREFIX} {MEM_POINTER} += {PREFIX} {MEM_OFFSET}\n\
                function {FUNC_LOAD}\n"
            )
        }

        Instruction::Random { dst, min, max } => {
            format!(
                "scoreboard players random {PREFIX} {} {min} {max}\n",
                register(dst)
            )
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
                "scoreboard players set {PREFIX} {MEM_POINTER} {addr}\n\
                scoreboard players operation {PREFIX} {MEM_POINTER} += {PREFIX} {MEM_OFFSET}\n\
                function {FUNC_STORE}\n"
            )
        }

        Instruction::Yield => {
            let an_label = ctx.new_anonymous_label();
            let an_block = ctx.get_label(switch.insert(an_label));
            format!(
                "scoreboard players set {PREFIX} {PROGRAM_COUNTER} {}",
                an_block.id()
            )
        }

        Instruction::Return => {
            switch = Some(ctx.new_anonymous_label());
            format!(
                "scoreboard players set {PREFIX} {MEM_POINTER} -1\n\
                function {FUNC_LOAD}\n\
                scoreboard players operation {PROGRAM_COUNTER} = {REG_R0}\n\
                function {FUNC_EXEC}\n
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

    ctx.get_label(label).push_str(&command);
    Ok(switch)
}
