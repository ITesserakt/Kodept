use crate::new_types::{Keyword, Symbol, TypeName};
use crate::prelude::TopLevelNode;
use derive_more::Constructor;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Module {
    Global {
        keyword: Keyword,
        id: TypeName,
        flow: Symbol,
        rest: Box<[TopLevelNode]>,
    },
    Ordinary {
        keyword: Keyword,
        id: TypeName,
        lbrace: Symbol,
        rest: Box<[TopLevelNode]>,
        rbrace: Symbol,
    },
}

#[derive(Debug, Clone, PartialEq, Constructor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct File(pub Box<[Module]>);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RLT(pub File);

impl Module {
    pub fn get_keyword(&self) -> &Keyword {
        match self {
            Module::Global { keyword, .. } => keyword,
            Module::Ordinary { keyword, .. } => keyword,
        }
    }
}

impl Located for Module {
    fn location(&self) -> CodePoint {
        self.get_keyword().location()
    }
}

impl Located for File {
    fn location(&self) -> CodePoint {
        CodePoint::new(0, 0)
    }
}

impl SpanBounds for File {
    fn bounds(&self) -> Span {
        CodePoint::single_point(0) + self.0.first().map(|it| it.bounds()) + self.0.last().map(|it| it.bounds())
    }
}

impl SpanBounds for Module {
    fn bounds(&self) -> Span {
        match self {
            Module::Global { keyword, rest, .. } => keyword.0 + rest.last().map(|it| it.bounds()),
            Module::Ordinary { keyword, rbrace, .. } => keyword.0 + rbrace.0
        }
    }
}
