use crate::engine::reporter::Reporter;
use crate::read_code_source::SyncSource;
use kodept_core::either::Either;
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

    fn extract_reports(self, sink: &mut Reporter<impl SyncSource>) -> Self::Output;
}

impl<T> ExtractReports<SingleExtractMarker> for T
where
    T: IntoSpannedReportMessage,
{
    type Output = ();

    #[inline]
    fn extract_reports(self, sink: &mut Reporter<impl SyncSource>) -> Self::Output {
        sink.report(self);
    }
}

impl<T, E, M> ExtractReports<(ResultExtractMarker, M)> for Result<T, E>
where
    E: ExtractReports<M>,
{
    type Output = ControlFlow<(), T>;

    #[inline]
    fn extract_reports(self, sink: &mut Reporter<impl SyncSource>) -> Self::Output {
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
    type Output = ControlFlow<<E::Output as Try>::Residual>;

    #[inline]
    fn extract_reports(self, sink: &mut Reporter<impl SyncSource>) -> Self::Output {
        self.into_iter().fold(Continue(()), |acc, next| {
            match next.extract_reports(sink).branch() {
                Continue(()) => acc,
                Break(e) => Break(e),
            }
        })
    }
}

impl<A, B, M1, M2, Output> ExtractReports<(EitherExtractMarker, M1, M2)> for Either<A, B>
where
    A: ExtractReports<M1, Output = Output>,
    B: ExtractReports<M2, Output = Output>,
{
    type Output = Output;

    #[inline]
    fn extract_reports(self, sink: &mut Reporter<impl SyncSource>) -> Self::Output {
        match self {
            Either::Left(left) => left.extract_reports(sink),
            Either::Right(right) => right.extract_reports(sink),
        }
    }
}
