#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::Identity;
use crate::graph::NodeId;
use crate::graph::{SyntaxTreeBuilder};
use crate::traits::{AsEnum, Linker, PopulateTree};
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
            ParamEnum::NonTy(x) => &x.name
        }
    }
}

impl PopulateTree for rlt::new_types::TypeName {
    type Output = TyName;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = TyName::uninit(context.get_chunk_located(self).to_string());

        builder.add_node(node).with_rlt(context, self).id()
    }
}

impl PopulateTree for rlt::TypedParameter {
    type Output = TyParam;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = TyParam::uninit(context.get_chunk_located(&self.id).to_string());
        builder
            .add_node(node)
            .with_children_from([&self.parameter_type], context)
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Type {
    type Output = Type;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Type::Reference(x) => x.convert(builder, context).cast(),
            rlt::Type::Tuple(x) => builder
                .add_node(ProdTy::uninit())
                .with_children_from(x.inner.iter().as_slice(), context)
                .with_rlt(context, self)
                .id()
                .cast()
        }
    }
}

impl PopulateTree for rlt::UntypedParameter {
    type Output = NonTyParam;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(NonTyParam::uninit(
                context.get_chunk_located(&self.id).to_string(),
            ))
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Parameter {
    type Output = Param;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Parameter::Typed(x) => x.convert(builder, context).cast(),
            rlt::Parameter::Untyped(x) => x.convert(builder, context).cast(),
        }
    }
}
