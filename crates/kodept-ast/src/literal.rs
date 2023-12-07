use crate::impl_identifiable;
use crate::node_id::NodeId;
use crate::traits::{IdProducer, Identifiable, Instantiable, IntoAst, Linker};
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
pub struct NumberLiteral {
    pub value: String,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CharLiteral {
    pub value: String,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct StringLiteral {
    pub value: String,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TupleLiteral {
    pub value: Vec<Literal>,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Literal {
    Number(NumberLiteral),
    Char(CharLiteral),
    String(StringLiteral),
    Tuple(TupleLiteral),
}

#[cfg(feature = "size-of")]
impl SizeOf for Literal {
    fn size_of_children(&self, context: &mut size_of::Context) {
        match self {
            Literal::Number(x) => x.size_of_children(context),
            Literal::Char(x) => x.size_of_children(context),
            Literal::String(x) => x.size_of_children(context),
            Literal::Tuple(x) => x.size_of_children(context),
        }
    }
}

impl_identifiable! {
    NumberLiteral, CharLiteral, StringLiteral, TupleLiteral
}
node_group!(family: Literal, nodes: [
    Literal, NumberLiteral, CharLiteral, StringLiteral, TupleLiteral
]);

impl Identifiable for Literal {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            Literal::Number(x) => x.get_id().cast(),
            Literal::Char(x) => x.get_id().cast(),
            Literal::String(x) => x.get_id().cast(),
            Literal::Tuple(x) => x.get_id().cast(),
        }
    }
}

impl IntoAst for rlt::Literal {
    type Output = Literal;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = match self {
            rlt::Literal::Binary(x)
            | rlt::Literal::Octal(x)
            | rlt::Literal::Hex(x)
            | rlt::Literal::Floating(x) => NumberLiteral {
                value: context.get_chunk_located(x).to_string(),
                id: context.next_id(),
            }
            .into(),
            rlt::Literal::Char(x) => CharLiteral {
                value: context.get_chunk_located(x).to_string(),
                id: context.next_id(),
            }
            .into(),
            rlt::Literal::String(x) => StringLiteral {
                value: context.get_chunk_located(x).to_string(),
                id: context.next_id(),
            }
            .into(),
            rlt::Literal::Tuple(x) => TupleLiteral {
                value: x.inner.iter().map(|it| it.construct(context)).collect(),
                id: context.next_id(),
            }
            .into(),
        };
        context.link(node, self)
    }
}

impl Instantiable for Literal {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            Literal::Number(x) => x.new_instance(context).into(),
            Literal::Char(x) => x.new_instance(context).into(),
            Literal::String(x) => x.new_instance(context).into(),
            Literal::Tuple(x) => x.new_instance(context).into(),
        }
    }
}

impl Instantiable for NumberLiteral {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            value: self.value.clone(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for CharLiteral {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            value: self.value.clone(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for StringLiteral {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            value: self.value.clone(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for TupleLiteral {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            value: self
                .value
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}
