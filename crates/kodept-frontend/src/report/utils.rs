use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use crate::Either;
use crate::engine::reporter::Reporter;
use crate::prelude::{GlobalReports, Source};
use crate::report::{Global, Reports};
use kodept_report::FileId;
use kodept_report::prelude::{IntoSpannedReportMessage, Reportable};
use kodept_report::report::Report;

pub struct SingleExtractMarker;
pub struct ResultExtractMarker;
pub struct IterExtractMarker;
pub struct EitherExtractMarker;

pub trait ExtractReports<Marker> {
    type Output;

    #[allow(private_bounds)]
    fn extract_reports_local<FileId, Impl>(
        self,
        file_id: FileId,
        sink: &Reports<Impl>,
    ) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId;

    fn extract_reports_global<Impl>(self, sink: &GlobalReports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>;

    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>;
}

pub(super) trait CorrectFileId: Sized + Clone {
    fn insert<Impl>(collector: &Reports<Impl>, message: Report<Self>)
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>;
}

impl CorrectFileId for () {
    fn insert<Impl>(collector: &Reports<Impl>, report: Report<Self>)
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        match collector {
            Reports::Disabled => {}
            Reports::Eager { settings, .. } => report
                .emit(&mut &**settings, &Global)
                .expect("Cannot emit diagnostics"),
            Reports::Lazy { global_sink, .. } => {
                let mut lock = global_sink.lock().unwrap_or_else(|it| it.into_inner());
                lock.push(report);
            }
        }
    }
}

impl CorrectFileId for FileId {
    fn insert<Impl>(collector: &Reports<Impl>, report: Report<Self>)
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        match collector {
            Reports::Disabled => {}
            Reports::Eager {
                settings, sources, ..
            } => report
                .emit(&mut &**settings, &**sources)
                .expect("Cannot emit diagnostics"),
            Reports::Lazy { local_sink, .. } => {
                let mut lock = local_sink.lock().unwrap_or_else(|it| it.into_inner());
                lock.push(report);
            }
        }
    }
}

impl<T> ExtractReports<SingleExtractMarker> for T
where
    T: IntoSpannedReportMessage,
{
    type Output = ();

    #[allow(private_bounds)]
    fn extract_reports_local<FileId, Impl>(
        self,
        file_id: FileId,
        sink: &Reports<Impl>,
    ) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId,
    {
        sink.report(file_id, self);
    }

    fn extract_reports_global<Impl>(self, sink: &GlobalReports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        sink.report(self);
    }

    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>
    {
        sink.report(self);
    }
}

impl<T, E, M> ExtractReports<(ResultExtractMarker, M)> for Result<T, E>
where
    E: ExtractReports<M>,
{
    type Output = ControlFlow<(), T>;

    #[allow(private_bounds)]
    fn extract_reports_local<FileId, Impl>(
        self,
        file_id: FileId,
        sink: &Reports<Impl>,
    ) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId,
    {
        match self {
            Ok(x) => Continue(x),
            Err(e) => {
                e.extract_reports_local(file_id, sink);
                Break(())
            }
        }
    }

    fn extract_reports_global<Impl>(self, sink: &GlobalReports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        match self {
            Ok(x) => Continue(x),
            Err(e) => {
                e.extract_reports_global(sink);
                Break(())
            }
        }
    }

    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>
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

    #[allow(private_bounds)]
    fn extract_reports_local<FileId, Impl>(
        self,
        file_id: FileId,
        sink: &Reports<Impl>,
    ) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId,
    {
        sink.report_many(file_id, self);
    }

    fn extract_reports_global<Impl>(self, sink: &GlobalReports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        sink.0.report_many((), self);
    }

    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>
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

    #[allow(private_bounds)]
    fn extract_reports_local<FileId, Impl>(
        self,
        file_id: FileId,
        sink: &Reports<Impl>,
    ) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId,
    {
        match self {
            Either::Left(left) => left.extract_reports_local(file_id, sink),
            Either::Right(right) => right.extract_reports_local(file_id, sink),
        }
    }

    fn extract_reports_global<Impl>(self, sink: &GlobalReports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        match self {
            Either::Left(left) => left.extract_reports_global(sink),
            Either::Right(right) => right.extract_reports_global(sink),
        }
    }

    fn extract_reports<Impl>(self, sink: &mut Reporter<Impl>) -> Self::Output
    where
        Impl: Send + Sync + 'static,
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>
    {
        match self {
            Either::Left(left) => left.extract_reports(sink),
            Either::Right(right) => right.extract_reports(sink),
        }
    }
}
