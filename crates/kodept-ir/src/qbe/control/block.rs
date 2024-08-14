use crate::qbe::constants::Value;
use crate::qbe::control::instruction::{phi, AnyInst};
use derive_more::{Constructor, Display};
use itertools::Itertools;
use std::fmt::Formatter;
use std::vec;

pub type Label = String;

#[derive(Debug, PartialEq)]
pub struct Block<'a> {
    label: Label,
    phi: Vec<phi>,
    instr: Vec<AnyInst<'a>>,
    jump: Option<Jump>,
}

#[derive(Display, Debug, PartialEq)]
pub enum Jump {
    #[display("jmp @{_0}")]
    Unconditional(Label),
    #[display("jnz {condition} @{on_false} @{on_true}")]
    Conditional {
        condition: Value,
        on_true: Label,
        on_false: Label,
    },
    #[display("ret")]
    ReturnVoid,
    #[display("ret {_0}")]
    Return(Value),
    #[display("hlt")]
    Terminate,
}

impl Display for Block<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "@{}", self.label)?;
        if !self.phi.is_empty() {
            writeln!(f, "\t{}", self.phi.iter().join("\n\t"))?;
        }
        if !self.instr.is_empty() {
            writeln!(f, "\t{}", self.instr.iter().join("\n\t"))?;
        }
        if let Some(jump) = &self.jump {
            write!(f, "\t{}", jump)?;
        }
        Ok(())
    }
}

impl<'a> Block<'a> {
    pub fn new(label: impl Into<Label>) -> Self {
        Self {
            label: label.into(),
            phi: vec![],
            instr: vec![],
            jump: None,
        }
    }

    pub fn with_jump(mut self, jump: Jump) -> Self {
        self.jump = Some(jump);
        self
    }

    pub fn with_phi(mut self, instr: phi) -> Self {
        self.phi.push(instr);
        self
    }

    pub fn with_instr(mut self, instr: impl Into<AnyInst<'a>>) -> Self {
        self.instr.push(instr.into());
        self
    }
}

impl Jump {
    pub fn ret(value: impl Into<Value>) -> Self {
        Self::Return(value.into())
    }

    pub fn uncond(label: impl Into<Label>) -> Self {
        Self::Unconditional(label.into())
    }

    pub fn cond(
        value: impl Into<Value>,
        on_zero: impl Into<Label>,
        on_else: impl Into<Label>,
    ) -> Self {
        Self::Conditional {
            condition: value.into(),
            on_true: on_zero.into(),
            on_false: on_else.into(),
        }
    }
}
