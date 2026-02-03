use crate::prelude::Erase;
use crate::properties::SourceSpan;
use bevy_ecs::prelude::{Entity, Query};
use bevy_ecs::system::SystemParam;
use kodept_core::code_point::Span;
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
