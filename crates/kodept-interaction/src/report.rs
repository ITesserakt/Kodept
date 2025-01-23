use bevy_ecs::event::EventRegistry;
use bevy_ecs::prelude::{Component, Event, EventWriter, Events, In, Single, World};
use bevy_ecs::system::SystemParam;
use extend::ext;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_core::file_name::FileName;
use kodept_report::error::report::{
    ad_hoc_message, IntoSpannedReportMessage, Report, SpannedReportMessage,
};
use kodept_report::FileId;

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct FileDescriptor {
    pub id: FileId,
    pub file_name: FileName,
}

#[derive(Debug, Event)]
pub struct ReportEmitted(Report);

#[derive(SystemParam)]
pub(crate) struct Reporter<'w> {
    file: Single<'w, &'static FileDescriptor>,
    events: EventWriter<'w, ReportEmitted>,
}

impl Reporter<'_> {
    pub fn report(&mut self, message: impl IntoSpannedReportMessage) {
        self.events
            .send(ReportEmitted(Report::from_message(self.file.id, message)));
    }

    pub fn report_ad_hoc<T>(&mut self, f: impl FnOnce() -> T)
    where
        T: SpannedReportMessage + 'static,
    {
        self.events.send(ReportEmitted(Report::from_message(
            self.file.id,
            ad_hoc_message(f),
        )));
    }
}

#[ext]
pub impl AST {
    fn prepare_reporting(&mut self, descriptor: FileDescriptor) {
        self.interact().immediate_with(
            descriptor,
            move |In(d): In<FileDescriptor>, world: &mut World| {
                world.spawn(d);

                EventRegistry::register_event::<ReportEmitted>(world);
            },
        );
    }

    fn extract_reports(&mut self, f: impl FnMut(Report)) {
        self.interact().immediate_exclusive(|w| {
            w.resource_mut::<Events<ReportEmitted>>()
                .drain()
                .map(|it| it.0)
                .for_each(f);
        });
    }
}
