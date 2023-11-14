use std::collections::HashMap;

mod generate;
mod parse;

#[derive(Debug)]
pub struct VirtualMachine<'a> {
    blocks: HashMap<&'a str, Vec<Instruction<'a>>>,
}

#[derive(Clone, Copy, Debug)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
}

#[derive(Clone, Copy, Debug)]
pub enum CmpOp {
    LessThan,
    GreaterThan,
    LessEq,
    GreaterEq,
    Equals,
    NotEquals,
}

#[derive(Clone, Copy, Debug)]
pub enum CalcOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Min,
    Max,
}

#[derive(Clone, Copy, Debug)]
pub enum ExprCmpIn {
    Value(i32),
    Range(Option<i32>, Option<i32>),
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction<'a> {
    RawCommand(&'a str),
    Debug { line: usize, info: &'a str },
    Log(&'a str),
    Move { dst: Register, src: Register },
    Set { dst: Register, value: i32 },
    Load { addr: i32 },
    Store { addr: i32 },
    Compare(CmpOp),
    CompareIn { not: bool, opr: ExprCmpIn },
    Branch(&'a str),
    BranchIf(&'a str),
    BranchIfNot(&'a str),
    Calculate(CalcOp),
    Random { dst: Register, min: i32, max: i32 },
    Call { offset_inc: i32, label: &'a str },
}
