//! This crate contains an algorithm to check (and infer) types based on
//! Hindley-Milner type system with constraints extension.
//! Also provides some structures, allowing to model those types.

use crate::r#type::TVar;

pub mod algorithm_u;
pub mod algorithm_w;
pub mod assumption;
pub mod constraint;
pub mod language;
pub mod substitution;
pub mod traits;
mod process;
pub mod r#type;

#[derive(Default, Clone, Debug)]
pub(crate) struct InferState {
    variable_index: usize,
}

impl InferState {
    pub(crate) fn new_var(&mut self) -> TVar {
        let result = TVar(self.variable_index);
        self.variable_index += 1;
        result
    }
}

pub const LOWER_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub const UPPER_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
