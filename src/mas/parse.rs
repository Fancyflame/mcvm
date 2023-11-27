use std::collections::{hash_map::Entry, HashMap};

use anyhow::{anyhow, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{i32 as parse_i32, space0, space1},
    combinator::{eof, map, opt, rest, value},
    error::{Error, ErrorKind},
    multi::{fold_many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple, Tuple},
    IResult, InputTakeAtPosition, Parser,
};

use super::{CalcOp, CmpOp, ExprCmpIn, Function, Instruction, Register, VirtualMachine};

impl<'a> VirtualMachine<'a> {
    pub fn parse(text: &'a str) -> Result<Self> {
        let mut blocks: HashMap<&str, Function> = HashMap::new();
        let mut current_label = None;

        for (line_index, line) in text.lines().enumerate() {
            if comment(line).is_ok() {
                continue;
            }

            let line_number = line_index + 1;

            let (_, (loi, ())) = pair(
                alt((parse_func_sig, parse_instruction(line_number))),
                comment,
            )(line)
            .map_err(|e| anyhow!("cannot parse code at line {line_number}: {e}"))?;

            match loi {
                FuncOrInst::Function(func) => {
                    current_label = Some(match blocks.entry(func.name) {
                        Entry::Occupied(_) => {
                            return Err(anyhow!("duplicated function `{}`", func.name))
                        }
                        Entry::Vacant(vac) => vac.insert(func),
                    });
                }

                FuncOrInst::Instruction(inst) => match &mut current_label {
                    Some(func) => func.instructions.push(inst),
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

enum FuncOrInst<'a> {
    Function(Function<'a>),
    Instruction(Instruction<'a>),
}

fn parse_func_sig(input: &str) -> IResult<&str, FuncOrInst> {
    map(
        preceded(
            space0,
            terminated(
                pair(
                    ident,
                    delimited(
                        pair(space0, tag("(")),
                        separated_list0(tag(","), delimited(space0, ident, space0)),
                        tag(")"),
                    ),
                ),
                preceded(space0, tag(":")),
            ),
        ),
        |(name, args)| {
            FuncOrInst::Function(Function {
                name,
                args,
                instructions: Vec::new(),
            })
        },
    )(input)
}

fn parse_instruction<'a>(
    line_number: usize,
) -> impl FnMut(&'a str) -> IResult<&'a str, FuncOrInst> {
    let cmd = command_format("cmd", (ls(expr_str),), |(cmd,)| {
        Instruction::RawCommand(cmd)
    });

    let mov = command_format("mov", (ls(register), ls(register)), |(dst, src)| {
        Instruction::Move { dst, src }
    });

    let set = command_format("set", (ls(register), ls(parse_i32)), |(dst, value)| {
        Instruction::Set { dst, value }
    });

    let load = command_format("load", (ls(parse_i32),), |(addr,)| Instruction::Load {
        addr,
    });

    let store = command_format("store", (ls(parse_i32),), |(addr,)| Instruction::Store {
        addr,
    });

    let cmp = command_format("cmp", (ls(cmp_operator),), |(operator,)| {
        Instruction::Compare(operator)
    });

    let cmpin = command_format("cmpin", (opt(ls(tag("not"))), ls(cmp_in)), |(not, expr)| {
        Instruction::CompareIn {
            not: not.is_some(),
            opr: expr,
        }
    });

    let b = command_format("b", (ls(ident),), |(label,)| Instruction::Branch(label));

    let bi = command_format("bi", (ls(ident),), |(label,)| Instruction::BranchIf(label));

    let calc = command_format("calc", (ls(calc_operator),), |(opr,)| {
        Instruction::Calculate(opr)
    });

    let rand = command_format(
        "rand",
        (ls(register), ls(parse_i32), ls(parse_i32)),
        |(dst, min, max)| Instruction::Random { dst, min, max },
    );

    let yield_now = command_format("yield", (ls(tag("yield")),), |_| Instruction::Yield);

    let call = command_format("call", (ls(parse_i32), ls(ident)), |(offset_inc, label)| {
        Instruction::Call { offset_inc, label }
    });

    let ret = command_format("ret", (), |()| Instruction::Return);

    let debug = command_format("debug", (ls(expr_str),), move |(info,)| {
        Instruction::Debug {
            line: line_number,
            info,
        }
    });

    let log = command_format("log", (ls(expr_str),), |(msg,)| Instruction::Log(msg));

    map(
        terminated(
            alt((
                cmd, mov, set, load, store, cmp, cmpin, b, bi, calc, rand, yield_now, call, ret,
                debug, log,
            )),
            comment,
        ),
        FuncOrInst::Instruction,
    )
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

fn ident(input: &str) -> IResult<&str, &str> {
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
        value(Register::R0, tag("R0")),
        value(Register::R1, tag("R1")),
        value(Register::R2, tag("R2")),
        value(Register::R3, tag("R3")),
    ))(input)
}

fn cmp_operator(input: &str) -> IResult<&str, CmpOp> {
    alt((
        value(CmpOp::Equals, tag("==")),
        value(CmpOp::NotEquals, tag("!=")),
        value(CmpOp::GreaterEq, tag(">=")),
        value(CmpOp::GreaterThan, tag(">")),
        value(CmpOp::LessEq, tag("<=")),
        value(CmpOp::LessThan, tag("<")),
    ))(input)
}

fn calc_operator(input: &str) -> IResult<&str, CalcOp> {
    alt((
        value(CalcOp::Add, tag("+")),
        value(CalcOp::Sub, tag("-")),
        value(CalcOp::Mul, tag("*")),
        value(CalcOp::Div, tag("/")),
        value(CalcOp::Rem, tag("%")),
        value(CalcOp::Min, tag("<")),
        value(CalcOp::Max, tag(">")),
    ))(input)
}

fn cmp_in(input: &str) -> IResult<&str, ExprCmpIn> {
    alt((
        map(
            separated_pair(opt(parse_i32), tag(".."), opt(parse_i32)),
            |(lb, ub)| ExprCmpIn::Range(lb, ub),
        ),
        map(parse_i32, |val| ExprCmpIn::Value(val)),
    ))(input)
}
