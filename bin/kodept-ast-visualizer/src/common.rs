use bevy_ecs::archetype::Archetype;
use bevy_ecs::component::{ComponentId, Components};
use bevy_ecs::prelude::*;
use bevy_ecs::system::{StaticSystemParam, SystemParam};
use bevy_utils::prelude::DebugName;
use kodept_ast::properties::Node;
use kodept_ast::resource::reflection::{DebugRegistry, DynDebug};
use kodept_systems::configs::OutputDirectory;
use kodept_systems::source::collection::SourceView;
use std::any::TypeId;
use std::io::Write;

#[derive(SystemParam)]
pub(crate) struct State<'w, 's, T: SystemParam + 'static> {
    pub output: Res<'w, OutputDirectory>,
    pub source: Res<'w, SourceView>,
    pub nodes: Query<'w, 's, EntityRef<'static>, With<Node>>,
    pub components: &'w Components,
    pub debug_registry: Option<Res<'w, DebugRegistry>>,
    pub extra: StaticSystemParam<'w, 's, T>,
}

impl<T: SystemParam + 'static> State<'_, '_, T>
where
    Self: StateOps,
{
    pub(crate) fn draw_node(
        &self,
        entity: EntityRef,
        buffer: &mut impl Write,
    ) -> std::io::Result<()> {
        let all_components = self.get_entity_components(entity.archetype());

        let components_debug_repr = all_components
            .map(|it| (entity.get_by_id(it), it))
            .filter_map(|(value, id)| Some((value.ok()?, self.components.get_info(id)?)))
            .filter_map(|(value, info)| {
                let type_id = info.type_id()?;
                #[allow(unsafe_code)]
                let debug_repr = match self.debug_registry.as_deref() {
                    None => unsafe { DebugRegistry::debug_dynamic_global(value, type_id) },
                    Some(registry) => unsafe { registry.debug_dynamic(value, type_id) },
                };
                Some((debug_repr, info.name(), info.mutable(), type_id))
            });

        StateOps::draw_node(self, entity.id(), components_debug_repr, buffer)
    }
}

pub trait StateOps {
    fn get_entity_components(&self, archetype: &Archetype) -> impl Iterator<Item = ComponentId>;
    fn draw_node<'a>(
        &self,
        id: Entity,
        properties: impl Iterator<Item = (DynDebug<'a>, DebugName, bool, TypeId)>,
        buffer: &mut impl Write,
    ) -> std::io::Result<()>;
}
