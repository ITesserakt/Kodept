use std::rc::Rc;
use kodept_ast::{BlockLevel, Body, BodyEnum, ExpressionBlock};
use kodept_inference::language::{Language, Literal};
use kodept_inference::r#type::PolymorphicType;
use crate::scope::ScopeError;
use crate::type_checker::InferError;
use crate::typing::store::{Store, StoreOps};
use crate::typing::traits::{Context, NodeWithType};

fn unit() -> Language {
    Language::Literal(Literal::Tuple(vec![]))
}
