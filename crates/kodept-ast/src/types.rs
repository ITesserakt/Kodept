use crate::graph::graph::SyntaxTree;
use crate::graph::traits::PopulateTree;
use crate::impl_identifiable_2;
use crate::node_id::NodeId;
use crate::traits::Linker;
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
    pub types: Vec<Type>,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SumType {
    pub types: Vec<Type>,
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
    pub parameter_type: Type,
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

impl_identifiable_2! { TypeName }

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
