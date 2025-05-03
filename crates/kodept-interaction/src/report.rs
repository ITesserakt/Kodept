use bevy_ecs::prelude::{Res, Resource, Single, World};
use bevy_ecs::system::SystemParam;
use extend::ext;
use kodept_ast::properties::Root;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_report::report::Report;
use kodept_report::traits::{ad_hoc_message, IntoSpannedReportMessage, SpannedReportMessage};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Resource)]
struct ReportWriter {
    sink: Box<dyn Fn(Report) + Send + Sync>,
    fail: AtomicBool,
}

#[derive(SystemParam)]
pub(crate) struct Reporter<'w> {
    root: Single<'w, &'static Root>,
    events: Res<'w, ReportWriter>,
}

impl Reporter<'_> {
    pub(crate) fn report(&self, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(self.root.associated_file.id(), message);
        self.events
            .fail
            .fetch_or(report.is_error(), Ordering::Relaxed);
        (self.events.sink)(report);
    }

    pub(crate) fn report_ad_hoc<T>(&self, f: impl FnOnce() -> T)
    where
        T: SpannedReportMessage,
    {
        let report = Report::from_message(self.root.associated_file.id(), ad_hoc_message(f));
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
        sink: impl Fn(Report) + Send + Sync + 'static,
    ) {
        self.interact()
            .immediate_exclusive(move |world: &mut World| {
                world.insert_resource(ReportWriter {
                    sink: Box::new(sink),
                    fail: AtomicBool::new(false),
                });
            });
    }
}
