use crate::graph::generic_node::GenericASTNode;
use crate::graph::traits::PopulateTree;
use crate::graph::{Identity, SyntaxTree};
use crate::node_id::NodeId;
use crate::traits::Linker;
use crate::{node, wrapper, Body, Operation};
use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

wrapper! {
    #[derive(Debug, PartialEq, From)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper CodeFlow {
        if(IfExpression) = GenericASTNode::If(x) => Some(x)
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
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
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ElifExpression {;
        pub condition: Identity<Operation>,
        pub body: Identity<Body>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ElseExpression {;
        pub body: Identity<Body>,
    }
}

impl PopulateTree for rlt::IfExpr {
    type Output = IfExpression;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
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

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
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

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
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
