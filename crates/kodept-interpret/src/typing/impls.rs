use std::rc::Rc;
use kodept_ast::Body;
use kodept_inference::language::Language;
use kodept_inference::r#type::PolymorphicType;
use crate::scope::ScopeError;
use crate::type_checker::InferError;
use crate::typing::traits::{Context, NodeWithType};

impl NodeWithType for Body {
    fn to_model(&self, _: Context) -> Result<Rc<Language>, ScopeError> {
        todo!()
    }

    fn type_of(&self, _: Context) -> Result<Rc<PolymorphicType>, InferError> {
        todo!()
    }
}