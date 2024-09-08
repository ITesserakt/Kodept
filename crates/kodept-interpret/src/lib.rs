use crate::conc_cache::ConcSecSlotMap;
use crate::operator_desugaring::{AccessExpander, BinaryOperatorExpander, UnaryOperatorExpander};
use kodept_ast::graph::GenericNodeKey;

// mod convert_model;
// mod node_family;
pub mod operator_desugaring;
mod scope;
pub mod semantic_analyzer;
mod symbol;
// pub mod type_checker;
mod conc_cache;

#[derive(Copy, Clone)]
pub struct Witness(());

impl Witness {
    pub fn fact(_: AccessExpander, _: BinaryOperatorExpander, _: UnaryOperatorExpander) -> Witness {
        Witness(())
    }

    pub fn prove<T>(self) -> T {
        panic!("Cannot prove contract")
    }
}

pub(crate) type Path = String;

pub type Cache<T> = ConcSecSlotMap<GenericNodeKey, T>;
