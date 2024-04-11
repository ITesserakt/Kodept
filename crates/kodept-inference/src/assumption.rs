use std::collections::HashMap;
use std::ops::Sub;
use std::rc::Rc;

use crate::language;
use crate::language::Language;
use crate::r#type::{MonomorphicType, PolymorphicType};
use crate::substitution::Substitutions;

type RLanguage = Rc<Language>;

#[derive(Clone, Debug, Default)]
pub struct Assumptions {
    value: HashMap<RLanguage, PolymorphicType>,
}

impl Assumptions {
    pub fn substitute_mut(&mut self, substitutions: &Substitutions) -> &mut Self {
        for t in self.value.values_mut() {
            *t = t.substitute(substitutions);
        }
        self
    }

    pub fn push(&mut self, expr: RLanguage, t: PolymorphicType) -> &mut Self {
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
    pub fn empty() -> Self {
        Self::default()
    }

    /// Removes all assumptions about specified var
    pub fn filter_all(&mut self, var: &language::Var) -> &mut Self {
        self.value
            .retain(|it, _| !matches!(it.as_ref(), Language::Var(v) if v == var));
        self
    }
}
