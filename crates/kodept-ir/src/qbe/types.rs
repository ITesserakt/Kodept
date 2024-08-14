use crate::qbe::defs::aggregate::TypeDef;
use derive_more::{Display, From};
use sealed::sealed;

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
#[display("w")]
pub struct Word;

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
#[display("l")]
pub struct Long;

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
#[display("s")]
pub struct Single;

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
#[display("d")]
pub struct Double;

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
#[display("b")]
pub struct Byte;

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
#[display("h")]
pub struct Half;

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone, From)]
pub enum BasicType {
    W(Word),
    L(Long),
    S(Single),
    D(Double),
}

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone, From)]
pub enum ExtendedType {
    B(Byte),
    H(Half),
}

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone, From)]
pub enum IntegerType {
    W(Word),
    L(Long),
}

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone, From)]
pub enum FloatingType {
    S(Single),
    D(Double),
}

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Void {}

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
#[display("m")]
pub struct SizeType;

#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Sign {
    #[display("s")]
    Signed,
    #[display("u")]
    Unsigned,
}

#[derive(Display, Debug, Eq, PartialEq, From)]
pub enum ABIType<'a> {
    #[from(forward)]
    Basic(BasicType),
    #[display("{_0}b")]
    Byte(Sign),
    #[display("{_0}h")]
    Half(Sign),
    #[display(":{}", _0.name())]
    Symbol(&'a TypeDef),
}

#[sealed]
pub trait ValueType {}

macro_rules! impl_value_type {
    ($t:ty) => {
        #[sealed::sealed]
        impl ValueType for $t {}
    };
    ($($t:ty$(,)?)*) => {
        $(impl_value_type!($t);)*
    }
}

impl_value_type![
    Word,
    Long,
    Single,
    Double,
    BasicType,
    ExtendedType,
    IntegerType,
    FloatingType,
    Void,
    SizeType,
    ABIType<'_>
];
