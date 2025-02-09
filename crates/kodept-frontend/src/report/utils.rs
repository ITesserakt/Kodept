use crate::prelude::Source;
use crate::report::{Global, Reports};
use kodept_report::prelude::Reportable;
use kodept_report::report::Report;
use kodept_report::FileId;

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
