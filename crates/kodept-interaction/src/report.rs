
use bevy_ecs::entity::{Entity, EntityHashMap};
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::prelude::{Res, Single};
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Query, ResMut, SystemParam};
use kodept_ast::interaction::Interaction;
use kodept_ast::prelude::Erase;
use kodept_ast::properties::{Node, Root};
use kodept_report::report::Report;
use kodept_report::traits::{ad_hoc_message, IntoSpannedReportMessage, SpannedReportMessage};
use kodept_report::FileId;

#[derive(Debug, Resource)]
struct RootsCache(EntityHashMap<Entity>);

#[derive(SystemParam)]
struct ReportFactory<'w, 's> {
    roots: Query<'w, 's, &'static Root>,
    nodes: Query<'w, 's, &'static ChildOf, With<Node>>,
    sink: Res<'w, Sink>,
    cache: ResMut<'w, RootsCache>
}

impl<'w, 's> ReportFactory<'w, 's> {
    fn in_context(&mut self, id: impl Erase) -> Reporter_<'_> {
        let root_id = self.cache.0.entry(id.erase().entity()).or_insert_with_key(|id| {
            self.nodes.root_ancestor(*id)
        });
        let root = self.roots.get(*root_id).expect("Root node of this hierarchy is not actually a root");
        Reporter_ { file_id: root.associated_file.id(), sink: &*self.sink }
    }
}

struct Reporter_<'w> {
    file_id: FileId,
    sink: &'w Sink
}

#[derive(Resource)]
struct Sink(Box<dyn Fn(Report) + Send + Sync + 'static>);

#[derive(SystemParam)]
pub(crate) struct Reporter<'w> {
    root: Single<'w, &'static Root>,
    sink: Res<'w, Sink>
}

impl<'w> Reporter_<'w> {
    pub(crate) fn report(&self, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(self.file_id, message);
        self.sink.0(report);
    }

    pub(crate) fn report_ad_hoc<T: SpannedReportMessage>(&self, f: impl FnOnce() -> T) {
        let report = Report::from_message(self.file_id, ad_hoc_message(f));
        self.sink.0(report);
    }
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
