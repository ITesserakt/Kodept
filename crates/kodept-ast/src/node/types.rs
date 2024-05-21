use derive_more::{From, Into};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::Identity;
use crate::graph::NodeId;
use crate::graph::{AnyNode, SyntaxTreeBuilder};
use crate::traits::{Linker, PopulateTree};
use crate::{node, wrapper};

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Type {
        type_name(TypeName) = AnyNode::TypeName(x) => x.into(),
        tuple(ProdType) = AnyNode::ProdType(x) => x.into(),
    }
}

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Parameter {
        typed(TypedParameter) = AnyNode::TypedParameter(x) => x.into(),
        untyped(UntypedParameter) = AnyNode::UntypedParameter(x) => x.into(),
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TypeName {
        pub name: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ProdType {;
        pub types: Vec<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TypedParameter {
        pub name: String,;
        pub parameter_type: Identity<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct UntypedParameter {
        pub name: String,;
    }
}

impl PopulateTree for rlt::new_types::TypeName {
    type Output = TypeName;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = TypeName::uninit(context.get_chunk_located(self).to_string());

        builder.add_node(node).with_rlt(context, self).id()
    }
}

impl PopulateTree for rlt::TypedParameter {
    type Output = TypedParameter;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = TypedParameter::uninit(context.get_chunk_located(&self.id).to_string());
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
                .add_node(ProdType::uninit())
                .with_children_from(x.inner.iter().as_slice(), context)
                .with_rlt(context, self)
                .id()
                .cast()
        }
    }
}

impl PopulateTree for rlt::UntypedParameter {
    type Output = UntypedParameter;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(UntypedParameter::uninit(
                context.get_chunk_located(&self.id).to_string(),
            ))
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Parameter {
    type Output = Parameter;

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
