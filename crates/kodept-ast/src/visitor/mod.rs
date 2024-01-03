use crate::visitor::visit_side::Skip;

pub mod visit_side;

pub type TraversingResult<E> = Result<(), Skip<E>>;
