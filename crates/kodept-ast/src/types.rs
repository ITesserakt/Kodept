use derive_more::{From, Into};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::GenericASTNode;
use crate::graph::Identity;
use crate::graph::NodeId;
use crate::graph::SyntaxTree;
use crate::traits::{Linker, PopulateTree};
use crate::{node, wrapper};

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Type {
        type_name(TypeName) = GenericASTNode::TypeName(x) => Some(x),
        tuple(ProdType) = GenericASTNode::ProdType(x) => Some(x),
        union(SumType) = GenericASTNode::SumType(x) => Some(x),
    }
}

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Parameter {
        typed(TypedParameter) = GenericASTNode::TypedParameter(x) => Some(x),
        untyped(UntypedParameter) = GenericASTNode::UntypedParameter(x) => Some(x),
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TypeName {
        pub name: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ProdType {;
        pub types: Vec<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct SumType {;
        pub types: Vec<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TypedParameter {
        pub name: String,;
        pub parameter_type: Identity<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct UntypedParameter {
        pub name: String,;
    }
}

impl PopulateTree for rlt::new_types::TypeName {
    type Output = TypeName;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = TypeName {
            name: context.get_chunk_located(self).to_string(),
            id: Default::default(),
        };

        builder.add_node(node).with_rlt(context, self).id()
    }
}

impl PopulateTree for rlt::TypedParameter {
    type Output = TypedParameter;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = TypedParameter {
            name: context.get_chunk_located(&self.id).to_string(),
            id: Default::default(),
        };
        builder
            .add_node(node)
            .with_children_from([&self.parameter_type], context)
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Type {
    type Output = Type;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Type::Reference(x) => x.convert(builder, context).cast(),
            rlt::Type::Tuple(x) => builder
                .add_node(ProdType {
                    id: Default::default(),
                })
                .with_children_from(x.inner.iter().as_slice(), context)
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Type::Union(x) => builder
                .add_node(SumType {
                    id: Default::default(),
                })
                .with_children_from(x.inner.iter().as_slice(), context)
                .with_rlt(context, self)
                .id()
                .cast(),
        }
    }
}

impl PopulateTree for rlt::UntypedParameter {
    type Output = UntypedParameter;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(UntypedParameter {
                name: context.get_chunk_located(&self.id).to_string(),
                id: Default::default(),
            })
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Parameter {
    type Output = Parameter;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Parameter::Typed(x) => x.convert(builder, context).cast(),
            rlt::Parameter::Untyped(x) => x.convert(builder, context).cast(),
        }
    }
}
