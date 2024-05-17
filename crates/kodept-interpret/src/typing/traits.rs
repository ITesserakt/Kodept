use kodept_ast::graph::{PermTkn, SyntaxTree};
use kodept_inference::language::Language;
use kodept_inference::r#type::PolymorphicType;

use crate::scope::ScopeTree;

#[derive(Debug, Copy, Clone)]
pub struct Context<'a> {
    scopes: &'a ScopeTree,
    ast: &'a SyntaxTree,
    token: &'a PermTkn
}

pub enum TypingProcess {
    Model(Language),
    Type(PolymorphicType)
}

pub trait NodeWithType {
    fn prepare(&self, cx: Context) -> TypingProcess;
}
