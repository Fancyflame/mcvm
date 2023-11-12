use std::collections::HashMap;

mod parse;

#[derive(Debug)]
pub struct VirtualMachine<'a> {
    blocks: HashMap<&'a str, Vec<Instruction<'a>>>,
}

#[derive(Clone, Copy, Debug)]
pub enum Register {
    L0,
    L1,
    L2,
    L3,
}

#[derive(Clone, Copy, Debug)]
pub enum CmpOp {
    LessThan,
    GreaterThan,
    LessEq,
    GreaterEq,
    Equals,
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction<'a> {
    RawCommand(&'a str),
    Copy {
        dst: Register,
        src: Register,
    },
    Set {
        dst: Register,
        value: i32,
    },
    Load {
        dst: Register,
        addr: i32,
    },
    Store {
        dst: Register,
        addr: i32,
    },
    Compare(CmpOp),
    CompareIn {
        lower_bound: Option<i32>,
        upper_bound: Option<i32>,
    },
    Branch(&'a str),
    BranchIf(&'a str),
    BranchIfNot(&'a str),
    Add,
    Call {
        offset_inc: i32,
        label: &'a str,
    },
}
