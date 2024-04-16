use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Sub;
use std::rc::Rc;

use crate::language::Language;
use crate::r#type::{MonomorphicType, PolymorphicType};
use crate::substitution::Substitutions;
use crate::{language, Environment};

type RLanguage = Rc<Language>;
type RPolymorphicType = Rc<PolymorphicType>;

#[derive(Clone, Debug, Default)]
pub struct Assumptions {
    value: HashMap<RLanguage, RPolymorphicType>,
}

impl Assumptions {
    pub fn substitute_mut(&mut self, substitutions: &Substitutions) -> &mut Self {
        for t in self.value.values_mut() {
            *t = Rc::new(t.substitute(substitutions));
        }
        self
    }

    pub fn push(&mut self, expr: RLanguage, t: RPolymorphicType) -> &mut Self {
        match self.value.entry(expr) {
            Entry::Occupied(slot) if slot.get() == &t => {}
            Entry::Occupied(mut slot) => {
                let mut env = Environment::default();
                let s0 = slot
                    .get()
                    .instantiate(&mut env)
                    .unify(&t.instantiate(&mut env))
                    .expect("Given assumption cannot be unified with the old one");
                slot.insert(Rc::new(t.substitute(&s0)));
            }
            Entry::Vacant(slot) => {
                slot.insert(t);
            }
        };
        self
    }

    #[must_use]
    pub fn generalize(&self, t: MonomorphicType) -> PolymorphicType {
        let occupied_types = self.value.iter().flat_map(|it| it.1.free_types()).collect();
        let vars = t.free_types().sub(&occupied_types);
        vars.into_iter()
            .fold(PolymorphicType::Monomorphic(t), |acc, next| {
                PolymorphicType::Binding {
                    bind: next.clone(),
                    binding_type: Box::new(acc),
                }
            })
    }

    #[must_use]
    pub fn get(&self, key: &Language) -> Option<&PolymorphicType> {
        self.value.get(key).map(|it| it.as_ref())
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

    pub fn merge(mut self, other: Assumptions) -> Self {
        for (key, value) in other.value {
            self.push(key, value);
        }
        self
    }
}

impl Display for Assumptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let out = self
            .value
            .iter()
            .map(|(key, value)| format!("{key} => {value}"))
            .intersperse(", ".to_string())
            .collect::<String>();

        write!(f, "[{out}]")
    }
}
