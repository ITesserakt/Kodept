use crate::r#type::TVar;

pub mod algorithm_u;
pub mod algorithm_w;
pub mod assumption;
pub mod language;
pub mod substitution;
pub mod r#type;
pub mod constraint;
pub mod traits;

#[derive(Default)]
pub struct Environment {
    variable_index: usize,
}

impl Environment {
    pub fn new_var(&mut self) -> TVar {
        let result = TVar(self.variable_index);
        self.variable_index += 1;
        result
    }
}

pub const LOWER_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub const UPPER_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
