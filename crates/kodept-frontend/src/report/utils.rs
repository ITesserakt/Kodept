use crate::prelude::{GlobalReports, Source};
use crate::report::{Global, Reports};
use crate::Execution;
use kodept_report::prelude::{IntoSpannedReportMessage, Reportable};
use kodept_report::report::Report;
use kodept_report::FileId;
use std::ops::ControlFlow::{Break, Continue};

pub struct SingleExtractMarker;
pub struct ResultExtractMarker;
pub struct IterExtractMarker;

pub trait ExtractReports<Marker> {
    type Output;

    #[allow(private_bounds)]
    fn extract_reports<FileId, Impl>(self, file_id: FileId, sink: &Reports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId;

    fn extract_reports_global<Impl>(self, sink: &GlobalReports<Impl>) -> Self::Output
    where
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
    type Output = Execution<()>;

    #[allow(private_bounds)]
    fn extract_reports<FileId, Impl>(self, file_id: FileId, sink: &Reports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId,
    {
        sink.report(file_id, self)
    }

    fn extract_reports_global<Impl>(self, sink: &GlobalReports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        sink.report(self)
    }
}

impl<T, E, M> ExtractReports<(ResultExtractMarker, M)> for Result<T, E>
where
    E: ExtractReports<M>,
{
    type Output = Execution<T>;

    #[allow(private_bounds)]
    fn extract_reports<FileId, Impl>(self, file_id: FileId, sink: &Reports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId,
    {
        match self {
            Ok(x) => Continue(x),
            Err(e) => {
                e.extract_reports(file_id, sink);
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
}

impl<I, E> ExtractReports<IterExtractMarker> for I
where
    I: IntoIterator<Item = E>,
    E: IntoSpannedReportMessage,
{
    type Output = Execution<()>;

    #[allow(private_bounds)]
    fn extract_reports<FileId, Impl>(self, file_id: FileId, sink: &Reports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
        FileId: CorrectFileId,
    {
        sink.report_many(file_id, self)
    }

    fn extract_reports_global<Impl>(self, sink: &GlobalReports<Impl>) -> Self::Output
    where
        Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
    {
        sink.0.report_many((), self)
    }
}
