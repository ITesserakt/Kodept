use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

use Constraint::{ExplicitInstance, ImplicitInstance};
use MonomorphicType::{Constant, Pointer, Primitive, Tuple, Var};

use crate::constraint::{Constraint, EqConstraint};
use crate::constraint::Constraint::Eq;
use crate::r#type::{MonomorphicType, PolymorphicType, TVar};
use crate::r#type::MonomorphicType::Fn;
use crate::substitution::Substitutions;

pub(crate) trait Substitutable {
    type Output;

    fn substitute(&self, subst: &Substitutions) -> Self::Output;
}

pub(crate) trait FreeTypeVars {
    fn free_types(self) -> HashSet<TVar>;
}

pub(crate) trait ActiveTVars {
    fn active_vars(self) -> HashSet<TVar>;
}

pub trait EnvironmentProvider<Key: Hash + std::cmp::Eq> {
    type Error;
    
    #[deprecated]
    fn get(&self, key: &Key) -> Option<Cow<PolymorphicType>> where Self::Error: Debug {
        self.maybe_get(key).unwrap()
    }
    
    fn maybe_get(&self, key: &Key) -> Result<Option<Cow<PolymorphicType>>, Self::Error>; 
}

// -------------------------------------------------------------------------------------------------

impl Substitutable for TVar {
    type Output = HashSet<TVar>;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        subst.get(self).unwrap_or(&Var(*self)).free_types()
    }
}

impl Substitutable for MonomorphicType {
    type Output = Self;

    fn substitute(&self, subst: &Substitutions) -> MonomorphicType {
        match self {
            Primitive(_) | Constant(_) => self.clone(),
            Var(x) => subst.get(x).unwrap_or(self).clone(),
            Fn(input, output) => Fn(
                Box::new(input.substitute(subst)),
                Box::new(output.substitute(subst)),
            ),
            Tuple(inner) => Tuple(crate::r#type::Tuple(inner.0.substitute(subst))),
            Pointer(inner) => Pointer(Box::new(inner.substitute(subst))),
        }
    }
}

impl Substitutable for PolymorphicType {
    type Output = Self;

    fn substitute(&self, subst: &Substitutions) -> PolymorphicType {
        let mut s = subst.clone();
        self.bindings.iter().for_each(|it| s.remove(it));
        Self {
            bindings: self.bindings.clone(),
            binding_type: self.binding_type.substitute(&s),
        }
    }
}

impl Substitutable for Constraint {
    type Output = Self;

    fn substitute(&self, subst: &Substitutions) -> Constraint {
        match self {
            Eq(EqConstraint { t1, t2 }) => Eq(EqConstraint {
                t1: t1.substitute(subst),
                t2: t2.substitute(subst),
            }),
            ExplicitInstance { t, s } => ExplicitInstance {
                t: t.substitute(subst),
                s: s.substitute(subst),
            },
            ImplicitInstance { t1, ctx, t2 } => ImplicitInstance {
                t1: t1.substitute(subst),
                ctx: ctx.substitute(subst),
                t2: t2.substitute(subst),
            },
        }
    }
}

impl<T: Substitutable> Substitutable for &T {
    type Output = T::Output;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        (*self).substitute(subst)
    }
}

impl<T: Substitutable> Substitutable for [T] {
    type Output = Vec<T::Output>;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        self.iter().map(|it| it.substitute(subst)).collect()
    }
}

impl<T: Substitutable<Output = HashSet<T>>> Substitutable for HashSet<T>
where
    T: Hash + std::cmp::Eq,
{
    type Output = HashSet<T>;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        self.iter()
            .map(|it| it.substitute(subst))
            .flatten()
            .collect()
    }
}

// -------------------------------------------------------------------------------------------------

impl FreeTypeVars for &TVar {
    fn free_types(self) -> HashSet<TVar> {
        HashSet::from([self.clone()])
    }
}

impl FreeTypeVars for &MonomorphicType {
    fn free_types(self) -> HashSet<TVar> {
        match self {
            Primitive(_) | Constant(_) => HashSet::new(),
            Var(x) => HashSet::from([x.clone()]),
            Fn(input, output) => &input.free_types() | &output.free_types(),
            Tuple(crate::r#type::Tuple(vec)) => vec.free_types(),
            Pointer(x) => x.free_types(),
        }
    }
}

impl FreeTypeVars for &PolymorphicType {
    fn free_types(self) -> HashSet<TVar> {
        let mut free = self.binding_type.free_types();
        self.bindings.iter().for_each(|it| {
            free.remove(it);
        });
        free
    }
}

impl<T: FreeTypeVars, I: IntoIterator<Item = T>> FreeTypeVars for I {
    fn free_types(self) -> HashSet<TVar> {
        self.into_iter()
            .fold(HashSet::new(), |acc, next| &acc | &next.free_types())
    }
}

// -------------------------------------------------------------------------------------------------

impl ActiveTVars for &Constraint {
    fn active_vars(self) -> HashSet<TVar> {
        match self {
            Eq(EqConstraint { t1, t2 }) => [t1.clone(), t2.clone()].free_types(),
            ExplicitInstance { t, s } => &t.free_types() | &s.free_types(),
            ImplicitInstance { t1, ctx, t2 } => {
                let set = &ctx.free_types() & &t2.free_types();
                &t1.free_types() | &set
            }
        }
    }
}

impl<T: ActiveTVars, I: IntoIterator<Item = T>> ActiveTVars for I {
    fn active_vars(self) -> HashSet<TVar> {
        self.into_iter()
            .fold(HashSet::new(), |acc, next| &acc | &next.active_vars())
    }
}
