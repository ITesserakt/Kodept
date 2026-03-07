use crate::ExportControlEvent;
use kodept_ast::syntax_tree::prelude::{AllNodesQuery, NodeSlot};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::system::Commands;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_systems::utils::LogSystemEx;

define_phase!(
    pub phase ExportAstPhase[ExportAstPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(system.trace_completion())
    }
);

fn system(all_nodes: AllNodesQuery, mut commands: Commands) {
    commands.trigger(ExportControlEvent::Start);

    for slot in all_nodes.iter() {
        match slot {
            NodeSlot::Root(x) => commands.trigger(ExportControlEvent::Root(x.id())),
            NodeSlot::Inner {
                parent_id,
                edge_metadata,
                node_ref,
            } => commands.trigger(ExportControlEvent::Inner {
                parent_id,
                metadata: edge_metadata,
                this_id: node_ref.id(),
            }),
        }
    }

    commands.trigger(ExportControlEvent::Finish);
}
