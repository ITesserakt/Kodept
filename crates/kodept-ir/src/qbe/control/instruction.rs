use self::macros::*;
use crate::qbe::constants::Value;
use crate::qbe::control::block::Label;
use crate::qbe::control::instruction::macros::{def_instruction, def_instructions};
use crate::qbe::types::{
    ABIType, BasicType, Double, IntegerType, Long, Single, SizeType, ValueType, Void,
};
use derive_more::derive::From;
use derive_more::Display;
use itertools::Itertools;
use sealed::sealed;
use std::fmt::Formatter;

#[derive(Debug, PartialEq)]
pub enum Count {
    Unspecified,
    N(usize),
}

#[sealed]
pub trait Instruction {
    const NAME: &'static str;
    const PARAM_COUNT: Count;
    type VType: ValueType;
}

#[derive(Display, Debug, PartialEq)]
pub enum Lvalue<T: ValueType> {
    #[display("{to} ={ty} ")]
    Assignment { to: Value, ty: T },
    #[display("")]
    Empty,
}

#[derive(Debug, PartialEq)]
pub struct Instr<T: ValueType, B: AsRef<[Value]>> {
    lvalue: Lvalue<T>,
    params: B,
}

impl<B: AsRef<[Value]>> Instr<Void, B> {
    pub fn stmt(params: B) -> Self {
        Self {
            lvalue: Lvalue::Empty,
            params,
        }
    }
}

mod macros {
    macro_rules! count {
        (0) => {
            Count::Unspecified
        };
        ($c:expr) => {
            Count::N($c)
        };
    }

    macro_rules! backing {
        (0) => {Box<[Value]>};
        ($c:expr) => {[Value; $c]};
    }

    macro_rules! ctor {
        (Void, $params:ty) => {
            pub fn stmt(params: $params) -> Self {
                Self(Instr::stmt(params))
            }
        };
        ($lvalue:ty, $params:ty) => {
            pub fn smtm(lvalue: Value, t: impl Into<$lvalue>, params: $params) -> Self {
                Self(Instr {
                    lvalue: Lvalue::Assignment { to: lvalue, ty: t.into() },
                    params,
                })
            }
        };
    }

    macro_rules! def_instruction {
        ($name:ident $t:ty [$count:expr]) => {
            #[derive(Debug, PartialEq)]
            #[allow(non_camel_case_types)]
            pub struct $name(Instr<$t, backing!($count)>);

            #[sealed]
            impl Instruction for $name {
                const NAME: &'static str = stringify!($name);
                #[allow(unsafe_code)]
                const PARAM_COUNT: Count = count!($count);
                type VType = $t;
            }

            impl Display for $name {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(
                        f,
                        "{}{} {}",
                        self.0.lvalue,
                        Self::NAME,
                        self.0.params.as_ref().iter().join(", ")
                    )
                }
            }

            impl $name {
                ctor!($t, backing!($count));
            }
        };
    }

    macro_rules! def_instructions {
        ({$($name:ident$(,)?)+} $t:ty [$count:expr]) => {
            $(
                def_instruction!($name $t [$count]);
            )+
        };
    }

    pub(crate) use {backing, count, ctor, def_instruction, def_instructions};
}

#[derive(Display, Debug, PartialEq, From)]
pub enum AnyInst<'a> {
    Add(add),
    Sub(sub),
    Div(div),
    Mul(mul),
    Neg(neg),
    UDiv(udiv),
    Rem(rem),
    URem(urem),
    Or(or),
    Xor(xor),
    And(and),
    Sar(sar),
    Shr(shr),
    Shl(shl),
    Stored(stored),
    Stores(stores),
    Storel(storel),
    Storew(storew),
    Storeh(storeh),
    Storeb(storeb),
    Loadd(loadd),
    Loads(loads),
    Loadl(loadl),
    Loadsw(loadsw),
    Loaduw(loaduw),
    Loadsh(loadsh),
    Loaduh(loaduh),
    Loadsb(loadsb),
    Loadub(loadub),
    Blit(blit),
    Alloc4(alloc4),
    Alloc8(alloc8),
    Alloc16(alloc16),
    Cast(cast),
    Copy(copy),
    Call(call<'a>),
    Vastart(vastart),
    Vaarg(vaarg),
}

def_instructions!({add, sub, div, mul} BasicType [2]);
def_instruction!(neg BasicType [1]);
def_instructions!({udiv, rem, urem} IntegerType [2]);
def_instructions!({or, xor, and} IntegerType [2]);
def_instructions!({sar, shr, shl} IntegerType [2]);

def_instructions!({stored, stores, storel, storew, storeh, storeb} Void [2]);
def_instruction!(loadd Double [1]);
def_instruction!(loads Single [1]);
def_instruction!(loadl Long [1]);
def_instructions!({loadsw, loaduw, loadsh, loaduh, loadsb, loadub} IntegerType [1]);
def_instruction!(blit Void [3]);
def_instructions!({alloc4, alloc8, alloc16} SizeType [1]);

def_instructions!({cast, copy} BasicType [1]);

def_instruction!(vastart Void [1]);
def_instruction!(vaarg BasicType [1]);

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub struct call<'a> {
    lvalue: Lvalue<ABIType<'a>>,
    fn_name: Value,
    args: Box<[Argument<'a>]>,
}

#[derive(Display, Debug, PartialEq)]
pub enum Argument<'a> {
    #[display("{ty} {value}")]
    Regular { ty: ABIType<'a>, value: Value },
    #[display("env %{_0}")]
    Environment(String),
    #[display("...")]
    Variadic,
}

impl<'a> Argument<'a> {
    pub fn regular(ty: impl Into<ABIType<'a>>, value: impl Into<Value>) -> Self {
        Self::Regular {
            ty: ty.into(),
            value: value.into(),
        }
    }
}

#[sealed]
impl<'a> Instruction for call<'a> {
    const NAME: &'static str = "call";
    const PARAM_COUNT: Count = Count::Unspecified;
    type VType = ABIType<'a>;
}

impl Display for call<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}call {}({})",
            self.lvalue,
            self.fn_name,
            self.args.iter().join(", ")
        )
    }
}

impl<'a> call<'a> {
    pub fn assignment(
        lvalue: Value,
        t: impl Into<ABIType<'a>>,
        name: Value,
        args: impl Into<Box<[Argument<'a>]>>,
    ) -> Self {
        Self {
            lvalue: Lvalue::Assignment { to: lvalue, ty: t.into() },
            fn_name: name,
            args: args.into(),
        }
    }

    pub fn stmt(name: Value, args: impl Into<Box<[Argument<'a>]>>) -> Self {
        Self {
            lvalue: Lvalue::Empty,
            fn_name: name,
            args: args.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub struct phi {
    lvalue: Lvalue<BasicType>,
    branches: Box<[Branch]>,
}

#[derive(Display, Debug, PartialEq)]
#[display("@{label} {value}")]
pub struct Branch {
    label: Label,
    value: Value,
}

#[sealed]
impl Instruction for phi {
    const NAME: &'static str = "phi";
    const PARAM_COUNT: Count = Count::Unspecified;
    type VType = BasicType;
}

impl Display for phi {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}phi ({})",
            self.lvalue,
            self.branches.iter().join(", ")
        )
    }
}
