use bevy_ecs::prelude::{Res, Single};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::SystemParam;
use kodept_ast::interaction::Interaction;
use kodept_ast::properties::Root;
use kodept_report::report::Report;
use kodept_report::traits::{ad_hoc_message, IntoSpannedReportMessage, SpannedReportMessage};

#[derive(Resource)]
struct Sink(Box<dyn Fn(Report) + Send + Sync + 'static>);

#[derive(SystemParam)]
pub(crate) struct Reporter<'w> {
    root: Single<'w, &'static Root>,
    sink: Res<'w, Sink>
}

impl Reporter<'_> {
    pub(crate) fn report(&self, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(self.root.associated_file.id(), message);
        self.sink.0(report);
    }

    pub(crate) fn report_ad_hoc<T>(&self, f: impl FnOnce() -> T)
    where
        T: SpannedReportMessage,
    {
        let report = Report::from_message(self.root.associated_file.id(), ad_hoc_message(f));
        self.sink.0(report);
    }
}

pub fn install_reporting_support(ctx: &mut Interaction, handler: impl Fn(Report) + Send + Sync + 'static) {
    ctx.immediate_exclusive(move |w| {
        w.insert_resource(Sink(Box::new(handler)));
    });
}
