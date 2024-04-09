use std::convert::Infallible;
use std::ops::{ControlFlow, FromResidual, Try};

use derive_more::From;

#[derive(Default)]
pub enum Skip<E> {
    Failed(E),
    #[default]
    Skipped,
}

#[derive(Debug, Default, From)]
pub enum Execution<E, R = ()> {
    Failed(E),
    #[from(ignore)]
    Completed(R),
    #[default]
    #[from(ignore)]
    Skipped,
}

pub struct ByteSize;

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
            Ok(_) => unreachable!(),
            Err(e) => Self::Failed(e.into()),
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

impl ByteSize {
    const QUANTITIES: [&'static str; 5] = ["B", "KB", "MB", "GB", "TB"];

    const fn compress_step(value: usize, index: usize) -> (usize, &'static str) {
        if value < 1024 || index + 1 >= Self::QUANTITIES.len() {
            (value, Self::QUANTITIES[index])
        } else {
            Self::compress_step(value / 1024, index + 1)
        }
    }

    pub const fn compress(value: usize) -> (usize, &'static str) {
        Self::compress_step(value, 0)
    }
}

pub(crate) mod macros {
    #[macro_export]
    macro_rules! yield_all {
        ($subroutine:expr) => {{
            let mut coro = std::pin::Pin::from($subroutine);
            loop {
                match coro.as_mut().resume(()) {
                    std::ops::CoroutineState::Yielded(it) => yield it,
                    std::ops::CoroutineState::Complete(_) => break,
                }
            }
        }};
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::utils::ByteSize;

    #[rstest]
    #[case(1, (1, "B"))]
    #[case(1024, (1, "KB"))]
    #[case(1024 * 1024 * 10, (10, "MB"))]
    #[case(1024 * 1024 * 1024 * 10, (10, "GB"))]
    #[case(1024 * 1024 * 1024 * 1024 * 1024, (1024, "TB"))]
    fn test_human_readable_bytes(#[case] input: usize, #[case] expected: (usize, &'static str)) {
        assert_eq!(ByteSize::compress(input), expected)
    }
}
