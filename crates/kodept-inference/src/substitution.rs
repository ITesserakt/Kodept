use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::mem::take;
use std::ops::Add;

use itertools::Itertools;

use crate::r#type::{MonomorphicType, TVar};

#[derive(Clone, PartialEq)]
#[repr(transparent)]
pub struct Substitutions(HashMap<TVar, MonomorphicType>);

impl Substitutions {
    #[must_use]
    pub fn compose(&self, other: &Substitutions) -> Self {
        let mut copy = self.clone();
        copy.merge(other.clone());
        copy
    }
    
    pub fn merge(&mut self, other: Substitutions) {
        let a: HashSet<_> = other
            .0
            .iter()
            .map(|(key, ty)| (key.clone(), ty & &*self))
            .collect();
        let b: HashSet<_> = take(&mut self.0)
            .into_iter()
            .map(|(key, ty)| (key, ty & &other))
            .collect();

        self.0 = b.union(&a).cloned().collect()
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
    
    pub fn from_iter<M: Into<MonomorphicType>>(iter: impl IntoIterator<Item = (TVar, M)>) -> Self {
        Self(HashMap::from_iter(iter.into_iter().map(|(a, b)| (a, b.into()))))
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

    fn add(mut self, rhs: Self) -> Self::Output {
        self.merge(rhs);
        self
    }
}

impl Add<&Substitutions> for Substitutions {
    type Output = Substitutions;

    fn add(mut self, rhs: &Substitutions) -> Self::Output {
        self.merge(rhs.clone());
        self
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

impl Debug for Substitutions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
