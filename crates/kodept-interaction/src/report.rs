use bevy_ecs::prelude::{Component, Res, Resource, Single, World};
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

#[derive(Resource)]
struct ReportWriter {
    sink: Box<dyn Fn(Report) + Send + Sync>,
}

#[derive(SystemParam)]
pub(crate) struct Reporter<'w> {
    file: Single<'w, &'static FileDescriptor>,
    events: Res<'w, ReportWriter>,
}

impl Reporter<'_> {
    pub(crate) fn report(&self, message: impl IntoSpannedReportMessage) {
        (self.events.sink)(Report::from_message(self.file.id, message));
    }

    pub(crate) fn report_ad_hoc<T>(&self, f: impl FnOnce() -> T)
    where
        T: SpannedReportMessage + 'static,
    {
        (self.events.sink)(Report::from_message(self.file.id, ad_hoc_message(f)));
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
                });
                world.spawn(descriptor);
            });
    }
}
