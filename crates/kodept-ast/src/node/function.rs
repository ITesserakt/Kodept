use derive_more::From;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::{Body, node, Parameter, Type, TypedParameter, wrapper};
use crate::graph::{GenericASTNode, NodeId};
use crate::graph::{Identity, SyntaxTreeBuilder};
use crate::traits::Linker;
use crate::traits::PopulateTree;

wrapper! {
    #[derive(Debug, PartialEq, From)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper FunctionDeclaration {
        bodied(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction(x) => Some(x),
        abstract(AbstractFunctionDeclaration) = GenericASTNode::AbstractFunction(x) => Some(x),
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct BodiedFunctionDeclaration {
        pub name: String,;
        pub parameters: Vec<Parameter>,
        pub return_type: Option<Type>,
        pub body: Identity<Body>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct AbstractFunctionDeclaration {
        pub name: String,;
        pub parameters: Vec<TypedParameter>,
        pub return_type: Option<Type>,
    }
}

impl PopulateTree for rlt::BodiedFunction {
    type Output = BodiedFunctionDeclaration;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
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
