use crate::Either;
use crate::engine::reporter::Reporter;
use crate::prelude::Source;
use kodept_report::prelude::{IntoSpannedReportMessage};
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

impl<I, E> ExtractReports<IterExtractMarker> for I
where
    I: IntoIterator<Item = E>,
    E: IntoSpannedReportMessage,
{
    type Output = ();

    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        // TODO: return using `try_for_each`
        self.into_iter().for_each(|x| x.extract_reports(sink))
    }
}

impl<A, B, M1, M2, Output> ExtractReports<(EitherExtractMarker, M1, M2)> for Either<A, B>
where
    A: ExtractReports<M1, Output = Output>,
    B: ExtractReports<M2, Output = Output>,
{
    type Output = Output;

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
