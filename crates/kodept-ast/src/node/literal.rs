#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::NodeId;
use crate::graph::{SyntaxTreeBuilder};
use crate::traits::Linker;
use crate::traits::PopulateTree;
use crate::{node, node_sub_enum, Operation};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum Lit {
        Num(NumLit),
        Char(CharLit),
        Str(StrLit),
        Tuple(TupleLit)
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct NumLit {
        pub value: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct CharLit {
        pub value: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct StrLit {
        pub value: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TupleLit {;
        pub value: Vec<Operation>,
    }
}

impl PopulateTree for rlt::Literal {
    type Output = Lit;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let mut from_num = |x| {
            builder
                .add_node(NumLit::uninit(
                    context.get_chunk_located(x).to_string(),
                ))
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
                .add_node(CharLit::uninit(
                    context.get_chunk_located(x).to_string(),
                ))
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Literal::String(x) => builder
                .add_node(StrLit::uninit(
                    context.get_chunk_located(x).to_string(),
                ))
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Literal::Tuple(x) => builder
                .add_node(TupleLit::uninit())
                .with_children_from(x.inner.as_ref(), context)
                .with_rlt(context, self)
                .id()
                .cast(),
        }
    }
}
