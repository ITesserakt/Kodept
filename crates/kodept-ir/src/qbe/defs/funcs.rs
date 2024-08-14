use std::fmt::Formatter;
use std::vec;
use derive_more::Display;
use itertools::Itertools;
use nonempty_collections::NEVec;
use crate::qbe::control::block::Block;
use crate::qbe::linkage::Linkage;
use crate::qbe::typedefs::Name;
use crate::qbe::types::ABIType;

#[derive(Display, Debug, Eq, PartialEq)]
pub enum Parameter<'a> {
    #[display("{ty} %{name}")]
    Regular {
        ty: ABIType<'a>,
        name: Name
    },
    #[display("env %{_0}")]
    Environment(Name),
    #[display("...")]
    Variadic
}

#[derive(Debug, PartialEq)]
pub struct Function<'a> {
    linkage: Linkage,
    return_ty: Option<ABIType<'a>>,
    name: Name,
    params: Vec<Parameter<'a>>,
    blocks: NEVec<Block<'a>>
}

impl<'a> Parameter<'a> {
    pub fn regular(ty: impl Into<ABIType<'a>>, name: impl Into<Name>) -> Self {
        Self::Regular { 
            ty: ty.into(),
            name: name.into()
        }
    }

    pub fn env(name: impl Into<Name>) -> Self {
        Self::Environment(name.into())
    }
}

impl Display for Function<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.linkage != Linkage::private() {
            write!(f, "{} ", self.linkage)?;
        }
        write!(f, "function ")?;
        if let Some(return_ty) = &self.return_ty {
            write!(f, "{return_ty} ")?;
        }
        writeln!(f, "${}({}) {{", self.name, self.params.iter().join(", "))?;
        writeln!(f, "{}", self.blocks.iter().into_iter().join("\n"))?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl<'a> Function<'a> {
    pub fn new(linkage: Linkage, name: impl Into<Name>, blocks: NEVec<Block<'a>>) -> Self {
        Self {
            linkage,
            return_ty: None,
            name: name.into(),
            params: vec![],
            blocks,
        }
    }

    pub fn with_return_type(mut self, ty: impl Into<ABIType<'a>>) -> Self {
        self.return_ty = Some(ty.into());
        self
    }

    pub fn with_param(mut self, param: Parameter<'a>) -> Self {
        self.params.push(param);
        self
    }
}
