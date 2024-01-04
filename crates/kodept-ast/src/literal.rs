use derive_more::{From, Into};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::GenericASTNode;
use crate::graph::NodeId;
use crate::graph::SyntaxTree;
use crate::traits::Linker;
use crate::traits::PopulateTree;
use crate::{node, wrapper};

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Literal {
        number(NumberLiteral) = GenericASTNode::Number(x) => Some(x),
        char(CharLiteral) = GenericASTNode::Char(x) => Some(x),
        string(StringLiteral) = GenericASTNode::String(x) => Some(x),
        tuple(TupleLiteral) = GenericASTNode::Tuple(x) => Some(x),
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct NumberLiteral {
        pub value: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct CharLiteral {
        pub value: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct StringLiteral {
        pub value: String,;
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TupleLiteral {;
        pub value: Vec<Literal>,
    }
}

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
