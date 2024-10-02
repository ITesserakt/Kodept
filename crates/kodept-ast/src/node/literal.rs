#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::SubSyntaxTree;
use crate::traits::PopulateTree;
use crate::{node, node_sub_enum, Operation};
use crate::interning::SharedStr;

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
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct NumLit {
        pub value: SharedStr,;
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct CharLit {
        pub value: SharedStr,;
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct StrLit {
        pub value: SharedStr,;
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TupleLit {;
        pub value: Vec<Operation>,
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::Literal {
    type Root = Lit;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        let from_num = |x| {
            SubSyntaxTree::new(
                NumLit::uninit(context.get_chunk_located(x)).with_rlt(self),
            )
        };

        match self {
            rlt::Literal::Binary(x) => from_num(x).cast(),
            rlt::Literal::Octal(x) => from_num(x).cast(),
            rlt::Literal::Hex(x) => from_num(x).cast(),
            rlt::Literal::Floating(x) => from_num(x).cast(),
            rlt::Literal::Char(x) => SubSyntaxTree::new(
                CharLit::uninit(context.get_chunk_located(x)).with_rlt(self),
            )
            .cast(),
            rlt::Literal::String(x) => SubSyntaxTree::new(
                StrLit::uninit(context.get_chunk_located(x)).with_rlt(self),
            )
            .cast(),
            rlt::Literal::Tuple(x) => SubSyntaxTree::new(TupleLit::uninit().with_rlt(self))
                .with_children_from(x.inner.as_ref(), context)
                .cast(),
        }
    }
}
