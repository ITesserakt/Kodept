use std::collections::HashMap;
use std::ops::Sub;

use crate::language;
use crate::language::Language;
use crate::r#type::{MonomorphicType, PolymorphicType};
use crate::substitution::Substitutions;

#[derive(Clone, Debug)]
pub struct Assumptions<'l> {
    value: HashMap<&'l Language, PolymorphicType>,
}

impl<'l> Assumptions<'l> {
    pub fn substitute_mut(&mut self, substitutions: &Substitutions) -> &mut Self {
        for t in self.value.values_mut() {
            *t = t.substitute(substitutions);
        }
        self
    }

    pub fn push(&mut self, expr: &'l Language, t: PolymorphicType) -> &mut Self {
        self.value.insert(expr, t);
        self
    }

    #[must_use]
    pub fn generalize(&self, t: MonomorphicType) -> PolymorphicType {
        let occupied_types = self.value.iter().flat_map(|it| it.1.free_types()).collect();
        let vars = t.free_types().sub(&occupied_types);
        vars.iter()
            .fold(PolymorphicType::Monomorphic(t), |acc, next| {
                PolymorphicType::Binding {
                    bind: next.clone(),
                    binding_type: Box::new(acc),
                }
            })
    }

    #[must_use]
    pub fn get(&self, key: &Language) -> Option<&PolymorphicType> {
        self.value.get(key)
    }

    #[must_use]
    pub fn empty() -> Assumptions<'l> {
        Assumptions {
            value: HashMap::new(),
        }
    }

    pub fn filter_all(&mut self, var: &language::Var) -> &mut Self {
        self.value
            .retain(|it, _| !matches!(it, Language::Var(v) if v == var));
        self
    }
}
