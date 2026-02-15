use crate::prelude::Erase;
use crate::properties::SourceSpan;
use kodept_core::code_point::Span;
use kodept_ecs::entity::Entity;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::system::{Query, SystemParam};
use tracing::warn;

#[derive(SystemParam)]
pub struct Spans<'w, 's> {
    spans: Query<'w, 's, &'static SourceSpan>,
}

impl Spans<'_, '_> {
    pub fn span_for(&self, id: impl Erase<Entity>) -> Span {
        let id = id.erase();
        match self.spans.get(id) {
            Ok(x) => x.0,
            Err(_) => {
                warn!("No span found for {id}, returning empty one");
                Span::default()
            }
        }
    }
}
