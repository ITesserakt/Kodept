use crate::graph::traits::PopulateTree;
use crate::graph::{Identity, SyntaxTree};
use crate::node_id::NodeId;
use crate::traits::{Identifiable, Linker};
use crate::{impl_identifiable_2, with_children, Body, Parameter, Type, TypedParameter};
use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::node_group;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BodiedFunctionDeclaration {
    id: NodeId<Self>,
    pub name: String,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct AbstractFunctionDeclaration {
    id: NodeId<Self>,
    pub name: String,
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum FunctionDeclaration {
    Abstract(AbstractFunctionDeclaration),
    Bodied(BodiedFunctionDeclaration),
}

node_group!(family: FunctionDeclaration, nodes: [
    FunctionDeclaration, BodiedFunctionDeclaration, AbstractFunctionDeclaration
]);
node_group!(family: BodiedFunctionDeclaration, nodes: [BodiedFunctionDeclaration, Body]);
node_group!(family: AbstractFunctionDeclaration, nodes: [AbstractFunctionDeclaration]);

impl_identifiable_2! { BodiedFunctionDeclaration, AbstractFunctionDeclaration }

with_children!(BodiedFunctionDeclaration => {
    pub parameters: Vec<Parameter>
    pub return_type: Option<Type>
    pub body: Identity<Body>
});

with_children!(AbstractFunctionDeclaration => {
    pub parameters: Vec<TypedParameter>
    pub return_type: Option<Type>
});

impl PopulateTree for rlt::BodiedFunction {
    type Output = BodiedFunctionDeclaration;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(BodiedFunctionDeclaration {
                id: Default::default(),
                name: context.get_chunk_located(&self.id).to_string(),
            })
            .with_children_from(self.return_type.as_ref().map(|x| &x.1), context)
            .with_children_from(self.params.iter().flat_map(|x| x.inner.as_ref()), context)
            .with_children_from([self.body.as_ref()], context)
            .with_rlt(context, self)
            .id()
    }
}

impl Identifiable for FunctionDeclaration {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            FunctionDeclaration::Abstract(x) => x.get_id().cast(),
            FunctionDeclaration::Bodied(x) => x.get_id().cast(),
        }
    }
}
