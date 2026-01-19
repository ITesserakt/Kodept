use std::ops::ControlFlow;

/// Port of stdlib's `Try` into stable Rust
pub trait Try {
    type Output;
    type Residual;

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

pub trait FnOutput {
    type Output;
}

impl<T> FnOutput for fn() -> T {
    type Output = T;
}

type Never = <fn() -> ! as FnOutput>::Output;

impl Try for () {
    type Output = ();
    type Residual = Never;

    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(())
    }
}

impl<T> Try for Option<T> {
    type Output = T;
    type Residual = Option<Never>;

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Some(x) => ControlFlow::Continue(x),
            None => ControlFlow::Break(None),
        }
    }
}

impl<C, B> Try for ControlFlow<B, C> {
    type Output = C;
    type Residual = B;

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        self
    }
}

impl<T, E> Try for Result<T, E> {
    type Output = T;
    type Residual = E;

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(x) => ControlFlow::Continue(x),
            Err(x) => ControlFlow::Break(x),
        }
    }
}
