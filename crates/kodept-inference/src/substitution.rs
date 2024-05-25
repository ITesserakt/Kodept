use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::Add;

use itertools::Itertools;

use crate::r#type::{MonomorphicType, TVar};

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct Substitutions(HashMap<TVar, MonomorphicType>);

impl Substitutions {
    #[must_use]
    pub fn compose(&self, other: &Substitutions) -> Self {
        let mapped: HashSet<_> = other
            .0
            .iter()
            .map(|(key, ty)| (key.clone(), ty & self))
            .collect();
        let set: HashSet<_> = self
            .0
            .iter()
            .map(|it| (it.0.clone(), it.1.clone()))
            .collect();
        Substitutions(mapped.union(&set).cloned().collect())
    }

    #[must_use]
    pub fn empty() -> Substitutions {
        Substitutions(HashMap::new())
    }

    #[must_use]
    pub fn single(from: TVar, to: MonomorphicType) -> Substitutions {
        Substitutions(HashMap::from([(from, to)]))
    }
    
    #[must_use]
    pub fn get(&self, key: &TVar) -> Option<&MonomorphicType> {
        self.0.get(key)
    }
    
    pub fn remove(&mut self, key: &TVar) {
        self.0.remove(key);
    }
    
    #[cfg(test)]
    pub(crate) fn into_inner(self) -> HashMap<TVar, MonomorphicType> {
        self.0
    }
}

impl Add for &Substitutions {
    type Output = Substitutions;

    fn add(self, rhs: Self) -> Self::Output {
        self.compose(rhs)
    }
}

impl Add for Substitutions {
    type Output = Substitutions;

    fn add(self, rhs: Self) -> Self::Output {
        self.compose(&rhs)
    }
}

impl Add<&Substitutions> for Substitutions {
    type Output = Substitutions;

    fn add(self, rhs: &Substitutions) -> Self::Output {
        self.compose(rhs)
    }
}

impl Add<Substitutions> for &Substitutions {
    type Output = Substitutions;

    fn add(self, rhs: Substitutions) -> Self::Output {
        self.compose(&rhs)
    }
}

impl Display for Substitutions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|it| format!("{} := {}", it.0, it.1))
                .join(", ")
        )
    }
}
