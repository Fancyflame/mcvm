use std::collections::{hash_map::Entry, HashMap};

use anyhow::{anyhow, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{i32 as parse_i32, space0, space1},
    combinator::{eof, map, opt, rest, value},
    error::{Error, ErrorKind},
    multi::fold_many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple, Tuple},
    IResult, InputTakeAtPosition, Parser,
};

use super::{CmpOp, Instruction, Register, VirtualMachine};

impl<'a> VirtualMachine<'a> {
    pub fn parse(text: &'a str) -> Result<Self> {
        let mut blocks: HashMap<&'a str, Vec<Instruction<'a>>> = HashMap::new();
        let mut current_label = None;

        for (line_index, line) in text.lines().enumerate() {
            if comment(line).is_ok() {
                continue;
            }

            let (_, (loi, ())) = pair(alt((parse_label, parse_instruction)), comment)(line)
                .map_err(|e| anyhow!("cannot parse code at line {}: {e}", line_index + 1))?;

            match loi {
                LabelOrInst::Label(label) => {
                    current_label = Some(match blocks.entry(label) {
                        Entry::Occupied(_) => return Err(anyhow!("duplicated label `{label}`")),
                        Entry::Vacant(vac) => vac.insert(Vec::new()),
                    });
                }
                LabelOrInst::Instruction(inst) => match &mut current_label {
                    Some(list) => list.push(inst),
                    None => {
                        return Err(anyhow!(
                            "instructions must under a label, you need to define a label first"
                        ))
                    }
                },
            }
        }

        Ok(VirtualMachine { blocks })
    }
}

enum LabelOrInst<'a> {
    Label(&'a str),
    Instruction(Instruction<'a>),
}

fn parse_label(input: &str) -> IResult<&str, LabelOrInst> {
    map(preceded(space0, pair(label, tag(":"))), |(label, _)| {
        LabelOrInst::Label(label)
    })(input)
}

fn parse_instruction(input: &str) -> IResult<&str, LabelOrInst> {
    let cmd = command_format("cmd", (ls(expr_str),), |(cmd,)| {
        Instruction::RawCommand(cmd)
    });

    let copy = command_format("copy", (ls(register), ls(register)), |(dst, src)| {
        Instruction::Copy { dst, src }
    });

    let set = command_format("set", (ls(register), ls(parse_i32)), |(dst, value)| {
        Instruction::Set { dst, value }
    });

    let load = command_format("load", (ls(register), ls(parse_i32)), |(dst, addr)| {
        Instruction::Load { dst, addr }
    });

    let store = command_format("store", (ls(register), ls(parse_i32)), |(dst, addr)| {
        Instruction::Store { dst, addr }
    });

    let cmp = command_format("cmp", (ls(operator),), |(operator,)| {
        Instruction::Compare(operator)
    });

    let cmpin = command_format("cmpin", (ls(range),), |((lower_bound, upper_bound),)| {
        Instruction::CompareIn {
            lower_bound,
            upper_bound,
        }
    });

    let b = command_format("b", (ls(label),), |(label,)| Instruction::Branch(label));

    let bi = command_format("bi", (ls(label),), |(label,)| Instruction::BranchIf(label));

    let bn = command_format("bn", (ls(label),), |(label,)| {
        Instruction::BranchIfNot(label)
    });

    let add = command_format("add", (), |()| Instruction::Add);

    let call = command_format("call", (ls(parse_i32), ls(label)), |(offset_inc, label)| {
        Instruction::Call { offset_inc, label }
    });

    map(
        terminated(
            alt((
                cmd, copy, set, load, store, cmp, cmpin, b, bi, bn, add, call,
            )),
            comment,
        ),
        LabelOrInst::Instruction,
    )(input)
}

fn command_format<'a, P, O, F>(
    t: &'a str,
    parser: P,
    mapper: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, Instruction>
where
    P: Tuple<&'a str, O, Error<&'a str>>,
    F: FnMut(O) -> Instruction<'a>,
{
    map(preceded(pair(space0, tag(t)), tuple(parser)), mapper)
}

// leading space
fn ls<'a, P, O>(parser: P) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    P: Parser<&'a str, O, Error<&'a str>>,
{
    preceded(space1, parser)
}

fn label(input: &str) -> IResult<&str, &str> {
    input.split_at_position1_complete(
        |c| !(c.is_alphanumeric() || c == '_'),
        ErrorKind::AlphaNumeric,
    )
}

fn comment(input: &str) -> IResult<&str, ()> {
    preceded(
        space0,
        alt((
            // # comment
            value((), pair(tag("#"), rest)),
            // end of input
            value((), eof),
        )),
    )(input)
}

fn expr_str(input: &str) -> IResult<&str, &str> {
    delimited(
        tag("\""),
        map(
            fold_many0(
                alt((is_not("\""), tag("\\\""))),
                || 1usize,
                |bound, slice: &str| bound + slice.len(),
            ),
            |bound| &input[1..bound],
        ),
        tag("\""),
    )(input)
}

fn register(input: &str) -> IResult<&str, Register> {
    alt((
        value(Register::L0, tag("L0")),
        value(Register::L1, tag("L1")),
        value(Register::L2, tag("L2")),
        value(Register::L3, tag("L3")),
    ))(input)
}

fn operator(input: &str) -> IResult<&str, CmpOp> {
    alt((
        value(CmpOp::Equals, tag("=")),
        value(CmpOp::GreaterEq, tag(">=")),
        value(CmpOp::GreaterThan, tag(">")),
        value(CmpOp::LessEq, tag("<=")),
        value(CmpOp::LessThan, tag("<")),
    ))(input)
}

fn range(input: &str) -> IResult<&str, (Option<i32>, Option<i32>)> {
    separated_pair(opt(parse_i32), tag(".."), opt(parse_i32))(input)
}
