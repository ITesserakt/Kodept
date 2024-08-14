use std::fmt::{Display, Formatter};
use itertools::Itertools;
use crate::qbe::defs::aggregate::TypeDef;
use crate::qbe::defs::data::DataDef;
use crate::qbe::defs::funcs::Function;

#[derive(Debug, PartialEq)]
pub struct Module<'a> {
    fns: Vec<Function<'a>>,
    types: Vec<TypeDef>,
    data: Vec<DataDef>
}

impl Display for Module<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.fns.is_empty() {
            writeln!(f, "{}", self.fns.iter().join("\n"))?;
        }
        if !self.data.is_empty() {
            writeln!(f, "{}", self.data.iter().join("\n"))?;
        }
        if !self.types.is_empty() {
            writeln!(f, "{}", self.types.iter().join("\n"))?;
        }
        Ok(())
    }
}

impl<'a> Module<'a> {
    pub const fn new() -> Self {
        Self {
            fns: vec![],
            types: vec![],
            data: vec![],
        }
    }

    pub fn with_fn(mut self, func: Function<'a>) -> Self {
        self.fns.push(func);
        self
    }

    pub fn with_data(mut self, data: DataDef) -> Self {
        self.data.push(data);
        self
    }

    pub fn with_type(mut self, ty: TypeDef) -> Self {
        self.types.push(ty);
        self
    }
}
