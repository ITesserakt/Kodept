use bevy_ecs::prelude::{Component, Res, Resource, Single, World};
use bevy_ecs::system::SystemParam;
use extend::ext;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_report::error::report::{
    ad_hoc_message, IntoSpannedReportMessage, Report, SpannedReportMessage,
};
use kodept_report::FileDescriptor;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
struct Wrapper(FileDescriptor);

#[derive(Resource)]
struct ReportWriter {
    sink: Box<dyn Fn(Report) + Send + Sync>,
    fail: AtomicBool,
}

#[derive(SystemParam)]
pub(crate) struct Reporter<'w> {
    file: Single<'w, &'static Wrapper>,
    events: Res<'w, ReportWriter>,
}

impl Reporter<'_> {
    pub(crate) fn report(&self, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(self.file.0.id, message);
        self.events
            .fail
            .fetch_or(report.is_error(), Ordering::Relaxed);
        (self.events.sink)(report);
    }

    pub(crate) fn report_ad_hoc<T>(&self, f: impl FnOnce() -> T)
    where
        T: SpannedReportMessage + 'static,
    {
        let report = Report::from_message(self.file.0.id, ad_hoc_message(f));
        self.events
            .fail
            .fetch_or(report.is_error(), Ordering::Relaxed);
        (self.events.sink)(report);
    }
}

#[ext]
pub impl AST {
    fn prepare_reporting(
        &mut self,
        descriptor: FileDescriptor,
        sink: impl Fn(Report) + Send + Sync + 'static,
    ) {
        self.interact()
            .immediate_exclusive(move |world: &mut World| {
                world.insert_resource(ReportWriter {
                    sink: Box::new(sink),
                    fail: AtomicBool::new(false),
                });
                world.spawn(Wrapper(descriptor));
            });
    }
}
