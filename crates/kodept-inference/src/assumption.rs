use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::Add;

use itertools::Itertools;

use crate::language::Var;
use crate::r#type::{MonomorphicType, PolymorphicType};
use crate::substitution::Substitutions;
use crate::traits::{EnvironmentProvider, Substitutable};

#[derive(PartialEq, Clone)]
#[repr(transparent)]
pub struct TypeTable<T, Name = Var>(HashMap<Name, T>)
where
    Name: Hash + Eq;

pub type AssumptionSet = TypeTable<Vec<MonomorphicType>, Var>;
pub type Environment = TypeTable<PolymorphicType, Var>;

pub trait TypeTableOps {
    fn merge(&mut self, other: Self);
}

// -------------------------------------------------------------------------------------------------

impl<K: Hash + Eq, V> Default for TypeTable<V, K> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<K: Hash + Eq, V> TypeTable<V, K> {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn remove<Q>(&mut self, key: &Q)
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.remove(key);
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys()
    }
}

impl<K, V> Add for TypeTable<V, K>
where
    K: Hash + Eq,
    Self: TypeTableOps,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.merge(rhs);
        self
    }
}

impl<K, V> Add<&TypeTable<V, K>> for TypeTable<V, K>
where
    K: Hash + Eq,
    Self: Clone + TypeTableOps,
{
    type Output = Self;

    fn add(mut self, rhs: &TypeTable<V, K>) -> Self::Output {
        self.merge(rhs.clone());
        self
    }
}

impl<K: Hash + Eq, V> Debug for TypeTable<V, K>
where
    Self: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

// -------------------------------------------------------------------------------------------------

impl AssumptionSet {
    pub fn push(&mut self, key: impl Into<Var>, value: impl Into<MonomorphicType>) {
        self.0.entry(key.into()).or_default().push(value.into())
    }

    pub fn get<K>(&self, key: &K) -> Cow<[MonomorphicType]>
    where
        Var: Borrow<K>,
        K: Hash + Eq,
    {
        match self.0.get(key) {
            None => Cow::Owned(vec![]),
            Some(x) => Cow::Borrowed(x),
        }
    }

    pub fn single(key: impl Into<Var>, value: impl Into<MonomorphicType>) -> Self {
        Self(HashMap::from([(key.into(), vec![value.into()])]))
    }

    pub fn merge_many(iter: impl IntoIterator<Item = AssumptionSet>) -> AssumptionSet {
        iter.into_iter()
            .fold(AssumptionSet::empty(), AssumptionSet::add)
    }
}

impl TypeTableOps for AssumptionSet {
    fn merge(&mut self, other: Self) {
        for (k, v) in other.0 {
            self.0.entry(k).or_default().extend(v)
        }
    }
}

impl Display for AssumptionSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|(k, v)| format!("{k} :: [{}]", v.iter().join(", ")))
                .join(", ")
        )
    }
}

// -------------------------------------------------------------------------------------------------

impl Environment {
    pub fn substitute_mut(&mut self, substitutions: &Substitutions) -> &mut Self {
        for t in self.0.values_mut() {
            *t = t.substitute(substitutions);
        }
        self
    }

    pub fn push(&mut self, key: impl Into<Var>, value: impl Into<PolymorphicType>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Var, &PolymorphicType)> {
        self.0.iter()
    }
}

impl TypeTableOps for Environment {
    fn merge(&mut self, other: Self) {
        self.0.extend(other.0)
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0.iter().map(|(k, v)| format!("{k} :: {v}")).join(", ")
        )
    }
}

impl EnvironmentProvider<Var> for Environment {
    type Error = Infallible;

    fn maybe_get(&self, key: &Var) -> Result<Option<Cow<PolymorphicType>>, Self::Error> {
        Ok(self.0.get(key).map(Cow::Borrowed))
    }
}
