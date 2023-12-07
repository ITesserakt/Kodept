use crate::node_id::NodeId;
use crate::traits::{IdProducer, Identifiable, Instantiable, IntoAst, Linker};
use crate::{impl_identifiable, Body, Operation};
use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::node_group;

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum CodeFlow {
    If(IfExpression),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct IfExpression {
    pub condition: Operation,
    pub body: Body,
    pub elif: Vec<ElifExpression>,
    pub el: Option<ElseExpression>,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ElifExpression {
    pub condition: Operation,
    pub body: Body,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ElseExpression {
    pub body: Body,
    id: NodeId<Self>,
}

impl_identifiable! {
    IfExpression, ElifExpression, ElseExpression
}
node_group!(family: IfExpression, nodes: [IfExpression, ElifExpression, ElseExpression]);

impl Identifiable for CodeFlow {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            CodeFlow::If(x) => x.get_id().cast(),
        }
    }
}

impl IntoAst for rlt::CodeFlow {
    type Output = CodeFlow;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = match self {
            rlt::CodeFlow::If(x) => x.construct(context).into(),
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::IfExpr {
    type Output = IfExpression;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = IfExpression {
            condition: self.condition.construct(context),
            body: self.body.construct(context),
            elif: self.elif.iter().map(|it| it.construct(context)).collect(),
            el: self.el.as_ref().map(|it| it.construct(context)),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::ElifExpr {
    type Output = ElifExpression;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = ElifExpression {
            condition: self.condition.construct(context),
            body: self.body.construct(context),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::ElseExpr {
    type Output = ElseExpression;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = ElseExpression {
            body: self.body.construct(context),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl Instantiable for CodeFlow {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            CodeFlow::If(x) => x.new_instance(context).into(),
        }
    }
}

impl Instantiable for IfExpression {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            condition: self.condition.new_instance(context),
            body: self.body.new_instance(context),
            elif: self
                .elif
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            el: self.el.as_ref().map(|it| it.new_instance(context)),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for ElifExpression {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            condition: self.condition.new_instance(context),
            body: self.body.new_instance(context),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for ElseExpression {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            body: self.body.new_instance(context),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}
