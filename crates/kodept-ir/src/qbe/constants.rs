use derive_more::Display;

use super::typedefs::Name;

#[derive(Display, Debug, PartialEq)]
pub enum Constant {
    Integer(i64),
    #[display("s_{_0}")]
    Float(f32),
    #[display("d_{_0}")]
    Double(f64),
    #[display("${_0}")]
    Symbol(Name)
}

#[derive(Display, Debug, PartialEq)]
pub enum DynConstant {
    Constant(Constant),
    #[display("thread ${_0}")]
    ThreadLocalSymbol(Name)
}

#[derive(Display, Debug, PartialEq)]
pub enum Value {
    DynConstant(DynConstant),
    #[display("%{_0}")]
    Value(Name)
}

impl Value {
    pub fn global(name: impl Into<Name>) -> Self {
        Self::DynConstant(DynConstant::Constant(Constant::Symbol(name.into())))
    }

    pub fn thread_global(name: impl Into<Name>) -> Self {
        Self::DynConstant(DynConstant::ThreadLocalSymbol(name.into()))
    }

    pub fn local(name: impl Into<Name>) -> Self {
        Self::Value(name.into())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::DynConstant(DynConstant::Constant(Constant::Integer(value)))
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::DynConstant(DynConstant::Constant(Constant::Float(value)))
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::DynConstant(DynConstant::Constant(Constant::Double(value)))
    }
}
