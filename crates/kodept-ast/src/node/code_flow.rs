use derive_more::From;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::{Body, node, Operation, wrapper};
use crate::graph::{GenericASTNode, SyntaxTreeBuilder};
use crate::graph::Identity;
use crate::graph::NodeId;
use crate::traits::{Linker, PopulateTree};

wrapper! {
    #[derive(Debug, PartialEq, From)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper CodeFlow {
        if(IfExpression) = GenericASTNode::If(x) => Some(x)
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct IfExpression {;
        pub condition: Identity<Operation>,
        pub body: Identity<Body>,
        pub elifs: Vec<ElifExpression>,
        pub elses: Option<ElseExpression>,
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
            .add_node(IfExpression {
                id: Default::default(),
            })
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
            .add_node(ElifExpression {
                id: Default::default(),
            })
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
            .add_node(ElseExpression {
                id: Default::default(),
            })
            .with_children_from([&self.body], context)
            .with_rlt(context, self)
            .id()
    }
}
