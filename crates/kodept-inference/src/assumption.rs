use std::borrow::{Borrow, Cow};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::Add;
use std::rc::Rc;

use itertools::Itertools;

use crate::language;
use crate::language::{Language, Var};
use crate::r#type::{MonomorphicType, PolymorphicType};
use crate::substitution::Substitutions;
use crate::traits::{FreeTypeVars, Substitutable};

type RLanguage = Rc<Language>;
type RPolymorphicType = Rc<PolymorphicType>;

type Name = Var;

#[derive(Debug, PartialEq, Clone)]
#[repr(transparent)]
pub struct Assumptions2(HashMap<Name, Vec<MonomorphicType>>);

impl Assumptions2 {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn push(&mut self, key: impl Into<Name>, value: impl Into<MonomorphicType>) {
        self.0.entry(key.into()).or_default().push(value.into())
    }

    pub fn remove<K>(&mut self, key: &K)
    where
        Name: Borrow<K>,
        K: Hash + Eq,
    {
        self.0.remove(key);
    }

    pub fn get<K>(&self, key: &K) -> Cow<[MonomorphicType]>
    where
        Name: Borrow<K>,
        K: Hash + Eq,
    {
        match self.0.get(key) {
            None => Cow::Owned(vec![]),
            Some(x) => Cow::Borrowed(x),
        }
    }

    pub fn single(key: impl Into<Name>, value: impl Into<MonomorphicType>) -> Self {
        Self(HashMap::from([(key.into(), vec![value.into()])]))
    }

    pub fn keys(&self) -> impl Iterator<Item = &Name> {
        self.0.keys()
    }

    pub fn merge(&mut self, other: Assumptions2) {
        self.0.extend(other.0)
    }

    pub fn merge_many(vec: impl Iterator<Item = Assumptions2>) -> Assumptions2 {
        vec.fold(Assumptions2::empty(), |mut acc, next| {
            acc.0.extend(next.0);
            acc
        })
    }
}

impl Add for Assumptions2 {
    type Output = Assumptions2;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.merge(rhs);
        self
    }
}

impl Add<&Assumptions2> for Assumptions2 {
    type Output = Assumptions2;

    fn add(mut self, rhs: &Assumptions2) -> Self::Output {
        self.merge(rhs.clone());
        self
    }
}

#[derive(Debug, Default)]
pub struct Assumptions {
    value: HashMap<Language, PolymorphicType>,
}

impl Assumptions {
    pub fn substitute_mut(&mut self, substitutions: &Substitutions) -> &mut Self {
        for t in self.value.values_mut() {
            *t = t.substitute(substitutions);
        }
        self
    }

    pub fn push(&mut self, expr: Language, t: impl Into<PolymorphicType>) {
        let t = t.into();
        match self.value.entry(expr) {
            Entry::Occupied(slot) if slot.get() == &t => {}
            Entry::Occupied(mut slot) => {
                slot.insert(t);
            }
            // I don't know is it applicable anymore
            // Entry::Occupied(mut slot) => {
            //     let mut env = Environment::default();
            //     let s0 = slot
            //         .get()
            //         .instantiate(&mut env)
            //         .unify(&t.instantiate(&mut env))
            //         .expect("Given assumption cannot be unified with the old one");
            //     slot.insert(Rc::new(t.substitute(&s0)));
            // }
            Entry::Vacant(slot) => {
                slot.insert(t);
            }
        };
    }

    #[must_use]
    pub fn generalize(&self, t: MonomorphicType) -> PolymorphicType {
        let occupied_types = self.value.iter().flat_map(|it| it.1.free_types()).collect();
        t.generalize(&occupied_types)
    }

    #[must_use]
    pub fn get(&self, key: &Language) -> Option<&PolymorphicType> {
        self.value.get(key)
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn remove(&mut self, key: &Language) -> Option<PolymorphicType> {
        self.value.remove(key)
    }

    /// Removes all assumptions about specified var
    pub fn retain_all(&mut self, var: &language::Var) -> &mut Self {
        self.value
            .retain(|it, _| !matches!(it, Language::Var(v) if v == var));
        self
    }

    pub fn merge(mut self, other: Assumptions) -> Self {
        for (key, value) in other.value {
            self.push(key, value);
        }
        self
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &Var> {
        self.value.keys().filter_map(|it| match it {
            Language::Var(x) => Some(x),
            _ => None
        })
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&Var, &PolymorphicType)> {
        self.value.iter().filter_map(|(k, v)| match k {
            Language::Var(x) => Some((x, v)),
            _ => None
        })
    }
}

impl Display for Assumptions {
    #[allow(unstable_name_collisions)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let out = self
            .value
            .iter()
            .map(|(key, value)| format!("{key} :: {value}"))
            .intersperse(", ".to_string())
            .collect::<String>();

        write!(f, "[{out}]")
    }
}
