use crate::graph::traits::PopulateTree;
use crate::graph::SyntaxTree;
use crate::node_id::NodeId;
use crate::traits::Linker;
use crate::{impl_identifiable_2, with_children};
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

impl_identifiable_2! {
    NumberLiteral, CharLiteral, StringLiteral, TupleLiteral
}
node_group!(family: Literal, nodes: [
    Literal, NumberLiteral, CharLiteral, StringLiteral, TupleLiteral
]);

with_children!(TupleLiteral => {
    pub value: Vec<Literal>
});

impl PopulateTree for rlt::Literal {
    type Output = Literal;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        let mut from_num = |x| {
            builder
                .add_node(NumberLiteral {
                    value: context.get_chunk_located(x).to_string(),
                    id: Default::default(),
                })
                .with_rlt(context, self)
                .id()
                .cast()
        };

        match self {
            rlt::Literal::Binary(x) => from_num(x),
            rlt::Literal::Octal(x) => from_num(x),
            rlt::Literal::Hex(x) => from_num(x),
            rlt::Literal::Floating(x) => from_num(x),
            rlt::Literal::Char(x) => builder
                .add_node(CharLiteral {
                    value: context.get_chunk_located(x).to_string(),
                    id: Default::default(),
                })
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Literal::String(x) => builder
                .add_node(StringLiteral {
                    value: context.get_chunk_located(x).to_string(),
                    id: Default::default(),
                })
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Literal::Tuple(x) => builder
                .add_node(TupleLiteral {
                    id: Default::default(),
                })
                .with_children_from(x.inner.as_ref(), context)
                .with_rlt(context, self)
                .id()
                .cast(),
        }
    }
}
