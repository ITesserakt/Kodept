use derive_more::From;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::Identity;
use crate::graph::tags::PRIMARY;
use crate::graph::NodeId;
use crate::graph::{AnyNode, SyntaxTreeBuilder};
use crate::traits::{Linker, PopulateTree};
use crate::{node, wrapper, Body, Operation};

wrapper! {
    #[derive(Debug, PartialEq, From)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper CodeFlow {
        if(IfExpression) = AnyNode::If(x) => x.into()
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct IfExpression {;
        pub condition: Identity<Operation> as PRIMARY,
        pub body: Identity<Body> as 0,
        pub elifs: Vec<ElifExpression> as 0,
        pub elses: Option<ElseExpression> as 0,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ElifExpression {;
        pub condition: Identity<Operation>,
        pub body: Identity<Body>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ElseExpression {;
        pub body: Identity<Body>,
    }
}

impl PopulateTree for rlt::IfExpr {
    type Output = IfExpression;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(IfExpression::uninit())
            .with_children_from([&self.condition], context)
            .with_children_from([&self.body], context)
            .with_children_from(self.elif.as_ref(), context)
            .with_children_from(self.el.as_slice(), context)
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::ElifExpr {
    type Output = ElifExpression;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(ElifExpression::uninit())
            .with_children_from([&self.condition], context)
            .with_children_from([&self.body], context)
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::ElseExpr {
    type Output = ElseExpression;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(ElseExpression::uninit())
            .with_children_from([&self.body], context)
            .with_rlt(context, self)
            .id()
    }
}
