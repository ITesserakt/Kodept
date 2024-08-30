#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::{Identity, SubSyntaxTree};
use crate::traits::{AsEnum, PopulateTree};
use crate::{node, node_sub_enum};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum Type {
        TyName(TyName),
        Tuple(ProdTy)
    }
}

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum Param {
        Ty(TyParam),
        NonTy(NonTyParam)
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TyName {
        pub name: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ProdTy {;
        pub types: Vec<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TyParam {
        pub name: String,;
        pub parameter_type: Identity<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct NonTyParam {
        pub name: String,;
    }
}

impl Param {
    pub fn name(&self) -> &str {
        match self.as_enum() {
            ParamEnum::Ty(x) => &x.name,
            ParamEnum::NonTy(x) => &x.name,
        }
    }
}

impl PopulateTree for rlt::new_types::TypeName {
    type Root = TyName;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        let node = TyName::uninit(context.get_chunk_located(self).to_string()).with_rlt(self);

        SubSyntaxTree::new(node)
    }
}

impl PopulateTree for rlt::TypedParameter {
    type Root = TyParam;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        let node = TyParam::uninit(context.get_chunk_located(&self.id).to_string()).with_rlt(self);
        SubSyntaxTree::new(node).with_children_from([&self.parameter_type], context)
    }
}

impl PopulateTree for rlt::Type {
    type Root = Type;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        match self {
            rlt::Type::Reference(x) => x.convert(context).cast(),
            rlt::Type::Tuple(x) => SubSyntaxTree::new(ProdTy::uninit().with_rlt(self))
                .with_children_from(x.inner.iter().as_slice(), context)
                .cast(),
        }
    }
}

impl PopulateTree for rlt::UntypedParameter {
    type Root = NonTyParam;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        SubSyntaxTree::new(
            NonTyParam::uninit(context.get_chunk_located(&self.id).to_string()).with_rlt(self),
        )
    }
}

impl PopulateTree for rlt::Parameter {
    type Root = Param;

    fn convert(
        &self,
        context: &impl CodeHolder,
    ) -> SubSyntaxTree<Self::Root> {
        match self {
            rlt::Parameter::Typed(x) => x.convert(context).cast(),
            rlt::Parameter::Untyped(x) => x.convert(context).cast(),
        }
    }
}
