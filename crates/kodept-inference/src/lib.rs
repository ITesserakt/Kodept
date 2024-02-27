use crate::r#type::Var;

pub mod algorithm_u;
pub mod algorithm_w;
pub mod assumption;
pub mod language;
pub mod substitution;
pub mod r#type;

#[derive(Default)]
pub struct Environment {
    variable_index: usize,
}

impl Environment {
    pub fn new_var(&mut self) -> Var {
        let result = Var(self.variable_index);
        self.variable_index += 1;
        result
    }
}

pub const LOWER_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub const UPPER_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
