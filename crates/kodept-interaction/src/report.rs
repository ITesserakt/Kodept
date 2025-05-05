use bevy_ecs::prelude::{Commands, Event, Single, Trigger};
use bevy_ecs::system::SystemParam;
use kodept_ast::interaction::Interaction;
use kodept_ast::properties::Root;
use kodept_report::report::Report;
use kodept_report::traits::{ad_hoc_message, IntoSpannedReportMessage, SpannedReportMessage};

#[derive(Debug, Event)]
struct ReportEvent(Report);

#[derive(SystemParam)]
pub(crate) struct Reporter<'w, 's> {
    root: Single<'w, &'static Root>,
    events: Commands<'w, 's>
}

impl Reporter<'_, '_> {
    pub(crate) fn report(&mut self, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(self.root.associated_file.id(), message);
        self.events.trigger(ReportEvent(report));
    }

    pub(crate) fn report_ad_hoc<T>(&mut self, f: impl FnOnce() -> T)
    where
        T: SpannedReportMessage,
    {
        let report = Report::from_message(self.root.associated_file.id(), ad_hoc_message(f));
        self.events.trigger(ReportEvent(report));
    }
}

pub fn install_reporting_support(ctx: &mut Interaction, mut handler: impl FnMut(Report) + Send + Sync + 'static) {
    ctx.immediate_exclusive(move |w| {
        w.add_observer(move |t: Trigger<ReportEvent>| {
            handler(t.0.clone())
        });
    });
}
