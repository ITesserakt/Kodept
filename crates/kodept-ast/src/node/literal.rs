#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::SubSyntaxTree;
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
    type Root = Lit;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        let from_num = |x| {
            SubSyntaxTree::new(
                NumLit::uninit(context.get_chunk_located(x).to_string()).with_rlt(self),
            )
        };

        match self {
            rlt::Literal::Binary(x) => from_num(x).cast(),
            rlt::Literal::Octal(x) => from_num(x).cast(),
            rlt::Literal::Hex(x) => from_num(x).cast(),
            rlt::Literal::Floating(x) => from_num(x).cast(),
            rlt::Literal::Char(x) => SubSyntaxTree::new(
                CharLit::uninit(context.get_chunk_located(x).to_string()).with_rlt(self),
            )
            .cast(),
            rlt::Literal::String(x) => SubSyntaxTree::new(
                StrLit::uninit(context.get_chunk_located(x).to_string()).with_rlt(self),
            )
            .cast(),
            rlt::Literal::Tuple(x) => SubSyntaxTree::new(TupleLit::uninit().with_rlt(self))
                .with_children_from(x.inner.as_ref(), context)
                .cast(),
        }
    }
}
