use crate::assumption::AssumptionSet;
use crate::constraint::Constraint;
use crate::r#type::MonomorphicType;
use crate::traits::{Executor, TypeInfer};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub(crate) struct PartialInfer(pub AssumptionSet, pub Vec<Constraint>, pub MonomorphicType);

pub struct Suspend<'a, E, T: TypeInfer<E>>(&'a E, PhantomData<T>);

pub enum Continuation<'a, E, T: TypeInfer<E>> {
    Empty,
    Custom(Box<ContFn<'a, E, T>>),
    Static(Box<Infer<'a, E, T>>)
}

type ContFn<'a, E, T> = dyn FnOnce(&mut T, <T as TypeInfer<E>>::Output) -> Infer<'a, E, T> + 'a;

#[must_use]
#[derive(Debug)]
pub enum Infer<'a, E, T: TypeInfer<E>> {
    Done {
        value: T::Output,
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

#[derive(Debug)]
pub struct DefaultExecutor<'a, E, T: TypeInfer<E>> {
    stack: Vec<Continuation<'a, E, T>>
}

impl<'a, E, T: TypeInfer<E>> Default for DefaultExecutor<'a, E, T> {
    fn default() -> Self {
        Self {
            stack: vec![]
        }
    }
}

impl<'a, E, T: TypeInfer<E>> Executor<'a, E, T> for &mut DefaultExecutor<'a, E, T> {
    type Error = T::Error;

    fn fold(self, state: &mut T, mut current: Infer<'a, E, T>) -> Result<T::Output, Self::Error> {
        self.stack.clear();
        
        loop {
            match current {
                Infer::Done { value } => match self.stack.pop() {
                    None => {
                        if cfg!(debug_assertions) {
                            assert!(self.stack.is_empty());
                            self.stack.clear();
                        }
                        return Ok(value)
                    },
                    Some(cont) => {
                        current = cont.call(state, value);
                    }
                }
                Infer::Failed { error, .. } => return Err(error),
                Infer::Suspended { expr, continuation } => {
                    self.stack.push(continuation);
                    current = state.apply(expr);
                }
            }
        }
    }
}

impl<'a, E, T: TypeInfer<E>> Debug for Continuation<'a, E, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Continuation::Empty => write!(f, "<empty>"),
            Continuation::Custom(_) => write!(f, "<closure>"),
            Continuation::Static(_) => write!(f, "<static>")
        }
    }
}

impl<'a, E, T: TypeInfer<E>> Continuation<'a, E, T> {
    pub fn custom(f: impl FnOnce(&mut T, T::Output) -> Infer<'a, E, T> + 'a) -> Self {
        Self::Custom(Box::new(f))
    }

    pub fn known(value: Infer<'a, E, T>) -> Self {
        Self::Static(Box::new(value))
    }

    pub fn call(self, state: &mut T, value: T::Output) -> Infer<'a, E, T> {
        match self {
            Continuation::Empty => Infer::Done { value },
            Continuation::Custom(f) => f(state, value),
            Continuation::Static(x) => *x,
        }
    }
}

impl<'a, E, T: TypeInfer<E>> Infer<'a, E, T> {
    pub fn suspend(input: &'a E) -> Suspend<'a, E, T> {
        Suspend(input, PhantomData)
    }

    pub fn done(output: T::Output) -> Self {
        Self::Done { value: output }
    }

    pub fn zip_with(self, other: Self, f: impl FnOnce(T::Output, T::Output) -> T::Output + 'a) -> Self
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

impl<'a, E, T: TypeInfer<E>> Suspend<'a, E, T> {
    pub fn map(self, f: impl FnOnce(&mut T, T::Output) -> T::Output + 'a) -> Infer<'a, E, T> {
        Infer::Suspended {
            expr: self.0,
            continuation: Continuation::custom(move |s, p| Infer::Done { value: f(s, p) }),
        }
    }

    pub fn and_then(
        self,
        f: impl FnOnce(&mut T, T::Output) -> Infer<'a, E, T> + 'a,
    ) -> Infer<'a, E, T> {
        Infer::Suspended {
            expr: self.0,
            continuation: Continuation::custom(f),
        }
    }

    pub fn try_map(
        self,
        f: impl FnOnce(&mut T, T::Output) -> Result<T::Output, T::Error> + 'a,
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
        f: impl FnOnce(&mut T, T::Output) -> Result<Infer<'a, E, T>, T::Error> + 'a,
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
    pub(crate) fn new<T>(
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
