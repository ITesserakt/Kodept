use crate::graph::traits::{Identifiable, PopulateTree};
use crate::graph::Identity;
use crate::graph::SyntaxTree;
use crate::node_id::NodeId;
use crate::traits::Linker;
use crate::{impl_identifiable_2, with_children};
use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TypeName {
    pub name: String,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ProdType {
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SumType {
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Type {
    Reference(TypeName),
    Tuple(ProdType),
    Union(SumType),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TypedParameter {
    pub name: String,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct UntypedParameter {
    pub name: String,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Parameter {
    Typed(TypedParameter),
    Untyped(UntypedParameter),
}

#[cfg(feature = "size-of")]
impl SizeOf for Type {
    fn size_of_children(&self, context: &mut size_of::Context) {
        match self {
            Type::Reference(x) => x.size_of_children(context),
            Type::Tuple(x) => x.size_of_children(context),
            Type::Union(x) => x.size_of_children(context),
        }
    }
}

impl_identifiable_2! { TypeName, ProdType, SumType, TypedParameter, UntypedParameter }

with_children!(ProdType => {
    pub types: Vec<Type>
});

with_children!(SumType => {
    pub types: Vec<Type>
});

with_children!(TypedParameter => {
    pub parameter_type: Identity<Type>
});

impl Identifiable for Parameter {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            Parameter::Typed(x) => x.get_id().cast(),
            Parameter::Untyped(x) => x.get_id().cast(),
        }
    }

    fn set_id(&mut self, value: NodeId<Self>) {
        match self {
            Parameter::Typed(x) => x.set_id(value.cast()),
            Parameter::Untyped(x) => x.set_id(value.cast()),
        }
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
