use crate::constraint::Constraint::{Eq, ExplicitInstance, ImplicitInstance};
use crate::constraint::{Constraint, EqConstraint};
use crate::substitution::Substitutions;
use crate::r#type::MonomorphicType::Fn;
use crate::r#type::{MonomorphicType, PolymorphicType, TVar};
use MonomorphicType::{Constant, Pointer, Primitive, Tuple, Var};
use kodept_interning::{GlobalInterner, Interned};
use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;

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

// -------------------------------------------------------------------------------------------------

impl Substitutable for TVar {
    type Output = HashSet<TVar>;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        subst
            .get(self)
            .unwrap_or(Var(*self).intern_owned())
            .free_types()
    }
}

impl Substitutable for MonomorphicType {
    type Output = Interned<MonomorphicType>;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        match self {
            Primitive(_) | Constant(_) => self.intern(),
            Var(x) => subst.get(x).unwrap_or(self.intern()),
            Fn(input, output) => {
                Fn(input.substitute(subst), output.substitute(subst)).intern_owned()
            }
            Tuple(inner) => Tuple(ChangeOutputType::wrap(inner).substitute(subst)).intern_owned(),
            Pointer(inner) => Pointer(inner.substitute(subst)).intern_owned(),
        }
    }
}

impl Substitutable for Interned<MonomorphicType> {
    type Output = Self;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        match self.0 {
            Primitive(_) | Constant(_) => *self,
            Var(x) => subst.get(x).unwrap_or(*self),
            Fn(input, output) => {
                Fn(input.substitute(subst), output.substitute(subst)).intern_owned()
            }
            Tuple(inner) => Tuple(ChangeOutputType::wrap(inner).substitute(subst)).intern_owned(),
            Pointer(inner) => Pointer(inner.substitute(subst)).intern_owned(),
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

#[repr(transparent)]
pub struct ChangeOutputType<I, T: ?Sized>(PhantomData<I>, T);

impl<T: Substitutable> Substitutable for [T] {
    type Output = Vec<T::Output>;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        self.iter().map(|it| it.substitute(subst)).collect()
    }
}

impl<I, T> ChangeOutputType<I, T> {
    pub fn wrap(slice: &[T]) -> ChangeOutputType<I, &[T]> {
        ChangeOutputType(PhantomData, slice)
    }
}

impl<I, T> Substitutable for ChangeOutputType<I, &[T]>
where
    T: Substitutable,
    I: FromIterator<T::Output>,
{
    type Output = I;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        self.1.iter().map(|it| it.substitute(subst)).collect()
    }
}

impl<T: Substitutable<Output = HashSet<T>>> Substitutable for HashSet<T>
where
    T: Hash + std::cmp::Eq,
{
    type Output = HashSet<T>;

    fn substitute(&self, subst: &Substitutions) -> Self::Output {
        self.iter().flat_map(|it| it.substitute(subst)).collect()
    }
}

// -------------------------------------------------------------------------------------------------

impl FreeTypeVars for &TVar {
    fn free_types(self) -> HashSet<TVar> {
        HashSet::from([*self])
    }
}

impl FreeTypeVars for &MonomorphicType {
    fn free_types(self) -> HashSet<TVar> {
        match self {
            Primitive(_) | Constant(_) => HashSet::new(),
            Var(x) => HashSet::from([*x]),
            Fn(input, output) => &input.free_types() | &output.free_types(),
            Tuple(vec) => vec.free_types(),
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

impl<T> FreeTypeVars for &[T]
where
    T: Deref,
    for<'a> &'a T::Target: FreeTypeVars,
{
    fn free_types(self) -> HashSet<TVar> {
        self.into_iter().fold(HashSet::new(), |acc, next| {
            &acc | &next.deref().free_types()
        })
    }
}

impl FreeTypeVars for &HashSet<TVar> {
    fn free_types(self) -> HashSet<TVar> {
        self.into_iter()
            .fold(HashSet::new(), |acc, next| &acc | &next.free_types())
    }
}

// -------------------------------------------------------------------------------------------------

impl ActiveTVars for &Constraint {
    fn active_vars(self) -> HashSet<TVar> {
        match self {
            Eq(EqConstraint { t1, t2 }) => [*t1, *t2].free_types(),
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
