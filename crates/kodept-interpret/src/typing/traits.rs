use crate::scope::{ScopeError, ScopeTree};
use crate::type_checker::InferError;
use kodept_ast::graph::{GhostToken, SyntaxTree};
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::Language;
use kodept_inference::r#type::PolymorphicType;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub struct Context<'a> {
    scopes: &'a ScopeTree,
    ast: &'a SyntaxTree,
    token: &'a GhostToken,
}

pub trait NodeWithType {
    fn to_model(&self, context: Context) -> Result<Rc<Language>, ScopeError>;
    fn type_of(&self, context: Context) -> Result<Rc<PolymorphicType>, InferError>;
    fn constraints(&self, context: Context) -> Result<Assumptions, InferError> {
        let mut a0 = Assumptions::empty();
        a0.push(self.to_model(context)?, self.type_of(context)?);
        Ok(a0)
    }
}
