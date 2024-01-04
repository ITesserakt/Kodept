use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use derive_more::Display;
use itertools::Itertools;

use crate::r#type::MonomorphicType;

#[derive(Debug, Clone, PartialEq, Display, Eq, Hash)]
#[display(fmt = "{substituted} <~ {replacement}")]
pub struct Substitution {
    pub(crate) substituted: MonomorphicType,
    pub(crate) replacement: MonomorphicType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Substitutions(pub(crate) HashSet<Substitution>);

impl Substitution {
    pub fn new<M1: Into<MonomorphicType>, M2: Into<MonomorphicType>>(to: M1, from: M2) -> Self {
        Self {
            substituted: from.into(),
            replacement: to.into(),
        }
    }
}

impl Substitutions {
    #[must_use]
    pub fn compose(&self, other: &Substitutions) -> Self {
        let a: HashSet<_> = other
            .0
            .iter()
            .map(|it| Substitution {
                substituted: it.substituted.clone(),
                replacement: it.replacement.substitute(self),
            })
            .collect();
        let b: HashSet<_> = self
            .0
            .iter()
            .map(|it| Substitution {
                substituted: it.substituted.clone(),
                replacement: it.replacement.substitute(other),
            })
            .collect();

        Substitutions(a.union(&b).cloned().collect())
    }

    #[must_use]
    pub fn empty() -> Substitutions {
        Substitutions(HashSet::new())
    }

    #[must_use]
    pub fn single(to: MonomorphicType, from: MonomorphicType) -> Substitutions {
        Substitutions(HashSet::from([Substitution {
            substituted: from,
            replacement: to,
        }]))
    }
}

impl Display for Substitutions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter().join(", "))
    }
}
