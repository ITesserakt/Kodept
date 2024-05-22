#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::{NodeId};
use crate::graph::{Identity, SyntaxTreeBuilder};
use crate::traits::Linker;
use crate::traits::PopulateTree;
use crate::{node, Body, Type, TyParam, node_sub_enum, Param};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum FnDecl {
        Body(BodyFnDecl),
        Abst(AbstFnDecl)
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct BodyFnDecl {
        pub name: String,;
        pub parameters: Vec<Param>,
        pub return_type: Option<Type>,
        pub body: Identity<Body>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct AbstFnDecl {
        pub name: String,;
        pub parameters: Vec<TyParam>,
        pub return_type: Option<Type>,
    }
}

impl PopulateTree for rlt::BodiedFunction {
    type Output = BodyFnDecl;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(BodyFnDecl::uninit(
                context.get_chunk_located(&self.id).to_string(),
            ))
            .with_children_from(self.return_type.as_ref().map(|x| &x.1), context)
            .with_children_from(self.params.iter().flat_map(|x| x.inner.as_ref()), context)
            .with_children_from([self.body.as_ref()], context)
            .with_rlt(context, self)
            .id()
    }
}
