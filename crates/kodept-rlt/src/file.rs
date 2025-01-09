use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use crate::new_types::{Keyword, Symbol, TypeName};
use crate::TopLevelNode;

#[derive(Debug, Clone, PartialEq)]
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
pub struct File(pub Box<[Module]>);

#[derive(Debug, Clone, PartialEq)]
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
