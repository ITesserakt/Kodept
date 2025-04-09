use crate::assumption::AssumptionSet;
use crate::constraint::Constraint;
use crate::r#type::MonomorphicType;
use crate::traits::PartialTypeInfer;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct PartialInfer(pub AssumptionSet, pub Vec<Constraint>, pub MonomorphicType);

pub struct Suspend<'a, E, T: PartialTypeInfer<E>>(&'a E, PhantomData<T>);

pub enum Continuation<'a, E, T: PartialTypeInfer<E>> {
    Empty,
    Custom(Box<Cont<'a, E, T>>)
}

type Cont<'a, E, T> = dyn FnOnce(&mut T, PartialInfer) -> Infer<'a, E, T> + 'a;

#[must_use]
#[derive(Debug)]
pub enum Infer<'a, E, T: PartialTypeInfer<E>> {
    Done {
        value: PartialInfer,
    },
    Failed {
        state: T,
        expr: &'a E,
        error: T::Error,
    },
    Suspended {
        expr: &'a E,
        continuation: Continuation<'a, E, T>,
    },
}

impl<'a, E, T: PartialTypeInfer<E>> Debug for Continuation<'a, E, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Continuation::Empty => write!(f, "<empty>"),
            Continuation::Custom(_) => write!(f, "<closure>")
        }
    }
}

impl<'a, E, T: PartialTypeInfer<E>> Continuation<'a, E, T> {
    pub fn custom(f: impl FnOnce(&mut T, PartialInfer) -> Infer<'a, E, T> + 'a) -> Self {
        Self::Custom(Box::new(f))
    }

    pub fn call(self, state: &mut T, value: PartialInfer) -> Infer<'a, E, T> {
        match self {
            Continuation::Empty => Infer::Done { value },
            Continuation::Custom(f) => f(state, value),
        }
    }
}

impl<'a, E, T: PartialTypeInfer<E>> Infer<'a, E, T> {
    pub fn suspend(input: &'a E) -> Suspend<'a, E, T> {
        Suspend(input, PhantomData)
    }

    pub fn done<I>(
        assumptions: AssumptionSet,
        constraints: impl IntoIterator<Item = I>,
        ty: impl Into<MonomorphicType>,
    ) -> Self
    where
        I: IntoIterator<Item = Constraint>,
    {
        Self::Done {
            value: PartialInfer::new(assumptions, constraints, ty),
        }
    }

    pub fn done_no_constraints(assumptions: AssumptionSet, ty: impl Into<MonomorphicType>) -> Self {
        Self::Done {
            value: PartialInfer(assumptions, vec![], ty.into()),
        }
    }

    pub fn done_empty(ty: impl Into<MonomorphicType>) -> Self {
        Self::Done {
            value: PartialInfer(AssumptionSet::empty(), vec![], ty.into()),
        }
    }

    #[inline]
    pub fn fold_with(
        self,
        state: &mut T,
        mut f: impl FnMut(&'a E, &PartialInfer),
    ) -> Result<PartialInfer, (&'a E, T, T::Error)> {
        let mut stack: Vec<(&E, Continuation<'a, E, T>)> = vec![];
        let mut current = self;
        loop {
            match current {
                Infer::Done { value } => {
                    if let Some((expr, cont)) = stack.pop() {
                        f(expr, &value);
                        current = cont.call(state, value);
                    } else {
                        return Ok(value);
                    }
                }
                Infer::Failed { state, expr, error } => return Err((expr, state, error)),
                Infer::Suspended { expr, continuation } => {
                    stack.push((expr, continuation));
                    current = state.apply(expr);
                }
            }
        }
    }

    #[inline]
    pub fn fold(self, state: &mut T) -> Result<PartialInfer, (&'a E, T, T::Error)> {
        self.fold_with(state, |_, _| {})
    }

    pub fn zip_with(self, other: Self, f: impl FnOnce(PartialInfer, PartialInfer) -> PartialInfer + 'a) -> Self
    where
        T: 'a,
    {
        match self {
            Infer::Done { value: a } => match other {
                Infer::Done { value: b } => Infer::Done { value: f(a, b) },
                e @ Infer::Failed { .. } => e,
                Infer::Suspended { expr, continuation } => {
                    T::suspend(expr).and_then(move |state, b| continuation.call(state, f(a, b)))
                }
            },
            e @ Infer::Failed { .. } => e,
            Infer::Suspended { expr, continuation } => {
                T::suspend(expr).and_then(move |state, a| continuation.call(state, a).zip_with(other, f))
            }
        }
    }
}

impl<'a, E, T: PartialTypeInfer<E>> Suspend<'a, E, T> {
    pub fn map(self, f: impl FnOnce(&mut T, PartialInfer) -> PartialInfer + 'a) -> Infer<'a, E, T> {
        Infer::Suspended {
            expr: self.0,
            continuation: Continuation::custom(move |s, p| Infer::Done { value: f(s, p) }),
        }
    }

    pub fn and_then(
        self,
        f: impl FnOnce(&mut T, PartialInfer) -> Infer<'a, E, T> + 'a,
    ) -> Infer<'a, E, T> {
        Infer::Suspended {
            expr: self.0,
            continuation: Continuation::custom(f),
        }
    }

    pub fn try_map(
        self,
        f: impl FnOnce(&mut T, PartialInfer) -> Result<PartialInfer, T::Error> + 'a,
    ) -> Infer<'a, E, T>
    where
        T: Clone,
    {
        Infer::Suspended {
            expr: self.0,
            continuation: Continuation::custom(move |s: &mut T, p| {
                let copy = s.clone();
                match f(s, p) {
                    Ok(x) => Infer::Done { value: x },
                    Err(e) => Infer::Failed {
                        state: copy,
                        expr: self.0,
                        error: e,
                    },
                }
            }),
        }
    }

    pub fn try_and_then(
        self,
        f: impl FnOnce(&mut T, PartialInfer) -> Result<Infer<'a, E, T>, T::Error> + 'a,
    ) -> Infer<'a, E, T>
    where
        T: Clone,
    {
        Infer::Suspended {
            expr: self.0,
            continuation: Continuation::custom(move |s: &mut T, p| {
                let copy = s.clone();
                f(s, p).unwrap_or_else(|error| Infer::Failed {
                    expr: self.0,
                    state: copy,
                    error,
                })
            }),
        }
    }
    
    #[inline]
    pub fn pure(self) -> Infer<'a, E, T> {
        Infer::Suspended {
            expr: self.0,
            continuation: Continuation::Empty
        }
    }
}

impl PartialInfer {
    pub fn new<T>(
        assumptions: AssumptionSet,
        constraints: impl IntoIterator<Item = T>,
        ty: impl Into<MonomorphicType>,
    ) -> Self
    where
        T: IntoIterator<Item = Constraint>,
    {
        Self(
            assumptions,
            constraints.into_iter().flatten().collect(),
            ty.into(),
        )
    }
}
