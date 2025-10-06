use crate::r#type::{MonomorphicType, TVar};
use crate::utils::JoinedDisplay;
use kodept_interning::{GlobalInterner, Interned};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::mem::take;
use std::ops::Add;

#[derive(Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Substitutions(HashMap<TVar, Interned<MonomorphicType>>);

impl Substitutions {
    pub fn merge(&mut self, other: &Substitutions) {
        let a: HashSet<_> = other
            .0
            .iter()
            .map(|(key, ty)| (*key, *ty & &*self))
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
    pub fn single(from: TVar, to: &MonomorphicType) -> Substitutions {
        Substitutions(HashMap::from([(from, to.intern())]))
    }

    #[must_use]
    pub fn get(&self, key: &TVar) -> Option<Interned<MonomorphicType>> {
        self.0.get(key).cloned()
    }

    pub fn remove(&mut self, key: &TVar) {
        self.0.remove(key);
    }

    #[cfg(test)]
    pub(crate) fn into_inner(self) -> HashMap<TVar, Interned<MonomorphicType>> {
        self.0
    }
}

impl FromIterator<(TVar, MonomorphicType)> for Substitutions {
    fn from_iter<T: IntoIterator<Item = (TVar, MonomorphicType)>>(iter: T) -> Self {
        Self(iter.into_iter().map(|it| (it.0, it.1.intern_owned())).collect())
    }
}

impl Add for &Substitutions {
    type Output = Substitutions;

    fn add(self, rhs: Self) -> Self::Output {
        let mut copy = self.clone();
        copy.merge(rhs);
        copy
    }
}

impl Add for Substitutions {
    type Output = Substitutions;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.merge(&rhs);
        self
    }
}

impl Add<&Substitutions> for Substitutions {
    type Output = Substitutions;

    fn add(mut self, rhs: &Substitutions) -> Self::Output {
        self.merge(rhs);
        self
    }
}

impl Add<Substitutions> for &Substitutions {
    type Output = Substitutions;

    fn add(self, rhs: Substitutions) -> Self::Output {
        let mut copy = self.clone();
        copy.merge(&rhs);
        copy
    }
}

impl Display for Substitutions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            JoinedDisplay::enumerate(self.0.iter().map(|it| format!("{} := {}", it.0, it.1)))
                .join()
        )
    }
}

impl Debug for Substitutions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
