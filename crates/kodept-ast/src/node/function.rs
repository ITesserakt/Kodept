#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::Identity;
use crate::graph::SubSyntaxTree;
use crate::traits::PopulateTree;
use crate::{node, node_sub_enum, Body, Param, TyParam, Type};
use crate::interning::SharedStr;

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum FnDecl {
        Body(BodyFnDecl),
        Abst(AbstFnDecl)
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct BodyFnDecl {
        pub name: SharedStr,;
        pub parameters: Vec<Param>,
        pub return_type: Option<Type>,
        pub body: Identity<Body>,
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct AbstFnDecl {
        pub name: SharedStr,;
        pub parameters: Vec<TyParam>,
        pub return_type: Option<Type>,
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::BodiedFunction {
    type Root = BodyFnDecl;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        SubSyntaxTree::new(
            BodyFnDecl::uninit(context.get_chunk_located(&self.id)).with_rlt(self),
        )
        .with_children_from(self.return_type.as_ref().map(|x| &x.1), context)
        .maybe_with_children_from(self.params.as_ref().map(|x| x.inner.as_ref()), context)
        .with_children_from([self.body.as_ref()], context)
    }
}
