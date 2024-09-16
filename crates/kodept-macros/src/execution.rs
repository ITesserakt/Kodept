use derive_more::From;
use kodept_ast::utils::Skip;
use std::convert::Infallible;
use std::ops::{ControlFlow, FromResidual, Try};

#[derive(Debug, Default, From)]
pub enum Execution<E, R = ()> {
    Failed(E),
    #[from(ignore)]
    Completed(R),
    #[default]
    #[from(ignore)]
    Skipped,
}

impl<E, R> Execution<E, R> {
    pub fn map<V>(self, f: impl Fn(R) -> V) -> Execution<E, V> {
        match self {
            Execution::Completed(it) => Execution::Completed(f(it)),
            Execution::Skipped => Execution::Skipped,
            Execution::Failed(e) => Execution::Failed(e),
        }
    }

    pub fn map_err<F>(self, f: impl Fn(E) -> F) -> Execution<F, R> {
        match self {
            Execution::Failed(e) => Execution::Failed(f(e)),
            Execution::Completed(it) => Execution::Completed(it),
            Execution::Skipped => Execution::Skipped,
        }
    }

    pub fn into_result(self) -> Result<R, E>
    where
        R: Default,
    {
        match self {
            Execution::Failed(e) => Err(e),
            Execution::Completed(it) => Ok(it),
            Execution::Skipped => Ok(R::default()),
        }
    }
}

impl<R> Execution<Infallible, R> {
    pub fn unwrap(self) -> Option<R> {
        match self {
            Execution::Completed(x) => Some(x),
            Execution::Skipped => None
        }
    }
}

impl<E, R> FromResidual for Execution<E, R> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Skip::Failed(e) => Self::Failed(e),
            Skip::Skipped => Self::Skipped,
        }
    }
}

impl<E1: Into<E2>, E2, R> FromResidual<Result<Infallible, E1>> for Execution<E2, R> {
    fn from_residual(residual: Result<Infallible, E1>) -> Self {
        match residual {
            Err(e) => Self::Failed(e.into()),
        }
    }
}

impl<E, R> FromResidual<Option<Infallible>> for Execution<E, R> {
    fn from_residual(residual: Option<Infallible>) -> Self {
        match residual {
            None => Self::Skipped,
        }
    }
}

impl<E, R> Try for Execution<E, R> {
    type Output = R;
    type Residual = Skip<E>;

    fn from_output(output: Self::Output) -> Self {
        Self::Completed(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Execution::Failed(e) => ControlFlow::Break(Skip::Failed(e)),
            Execution::Completed(it) => ControlFlow::Continue(it),
            Execution::Skipped => ControlFlow::Break(Skip::Skipped),
        }
    }
}