use crate::visitor::visit_side::{Skip, VisitSide};
use crate::*;

pub mod visit_side;

pub type TraversingResult<E> = Result<(), Skip<E>>;
