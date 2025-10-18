use crate::Either;
use crate::engine::reporter::Reporter;
use crate::prelude::Source;
use kodept_core::try_port::Try;
use kodept_report::prelude::IntoSpannedReportMessage;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};

pub struct SingleExtractMarker;
pub struct ResultExtractMarker;
pub struct IterExtractMarker;
pub struct EitherExtractMarker;

pub trait ExtractReports<Marker> {
    type Output;

    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>;
}

impl<T> ExtractReports<SingleExtractMarker> for T
where
    T: IntoSpannedReportMessage,
{
    type Output = ();

    #[inline]
    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        sink.report(self);
    }
}

impl<T, E, M> ExtractReports<(ResultExtractMarker, M)> for Result<T, E>
where
    E: ExtractReports<M>,
{
    type Output = ControlFlow<(), T>;

    #[inline]
    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        match self {
            Ok(x) => Continue(x),
            Err(e) => {
                e.extract_reports(sink);
                Break(())
            }
        }
    }
}

impl<I, E, M> ExtractReports<(IterExtractMarker, M)> for I
where
    I: IntoIterator<Item = E>,
    E: ExtractReports<M, Output: Try<Output = ()>>,
{
    type Output = ControlFlow<<E::Output as Try>::Residual, ()>;

    #[inline]
    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        let mut error = Continue(());
        for item in self {
            if let Break(e) = item.extract_reports(sink).branch() {
                error = Break(e);
            }
        }
        error
    }
}

impl<A, B, M1, M2, Output> ExtractReports<(EitherExtractMarker, M1, M2)> for Either<A, B>
where
    A: ExtractReports<M1, Output = Output>,
    B: ExtractReports<M2, Output = Output>,
{
    type Output = Output;

    #[inline]
    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        match self {
            Either::Left(left) => left.extract_reports(sink),
            Either::Right(right) => right.extract_reports(sink),
        }
    }
}
