#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::tags::PRIMARY;
use crate::graph::{Identity, SubSyntaxTree};
use crate::traits::PopulateTree;
use crate::{node, node_sub_enum, Body, Operation};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    pub enum CodeFlow {
        If(IfExpr)
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct IfExpr {;
        pub condition: Identity<Operation> as PRIMARY,
        pub body: Identity<Body> as 0,
        pub elifs: Vec<ElifExpr> as 0,
        pub elses: Option<ElseExpr> as 0,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ElifExpr {;
        pub condition: Identity<Operation>,
        pub body: Identity<Body>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ElseExpr {;
        pub body: Identity<Body>,
    }
}

impl PopulateTree for rlt::IfExpr {
    type Root = IfExpr;

    fn convert(&self, context: &mut impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        SubSyntaxTree::new(IfExpr::uninit().with_rlt(self))
            .with_children_from([&self.condition], context)
            .with_children_from([&self.body], context)
            .with_children_from(self.elif.as_ref(), context)
            .with_children_from(self.el.as_slice(), context)
    }
}

impl PopulateTree for rlt::ElifExpr {
    type Root = ElifExpr;

    fn convert(&self, context: &mut impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        SubSyntaxTree::new(ElifExpr::uninit().with_rlt(self))
            .with_children_from([&self.condition], context)
            .with_children_from([&self.body], context)
    }
}

impl PopulateTree for rlt::ElseExpr {
    type Root = ElseExpr;

    fn convert(&self, context: &mut impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        SubSyntaxTree::new(ElseExpr::uninit().with_rlt(self))
            .with_children_from([&self.body], context)
    }
}
