use kodept_ast::properties::Node;
use kodept_ast::resource::reflection::{DebugRegistry, DynDebug};
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{ComponentId, ComponentInfo, Components};
use kodept_ecs::entity::Entity;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::With;
use kodept_ecs::system::{Query, Res, StaticSystemParam, SystemParam};
use kodept_ecs::utils::DebugName;
use kodept_ecs::world::EntityRef;
use kodept_systems::configs::OutputDirectory;
use kodept_systems::source::collection::SourceView;
use std::any::TypeId;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, DerefMut};

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
                Some((debug_repr, info.name(), info.mutable(), info))
            });

        StateOps::draw_node(self, entity.id(), components_debug_repr, buffer)
    }

    pub(crate) fn provide_output_file(
        &mut self,
        extension: impl AsRef<OsStr>,
    ) -> std::io::Result<File> {
        self.output.create_missing_folders()?;
        let source_descriptor = self.source.describe();
        let filename = source_descriptor.name();
        let output_path = self
            .output
            .get_path_for_source(filename, extension.as_ref())?;

        File::create(output_path)
    }
}

impl<'w, 's, T: SystemParam + 'static> Deref for State<'w, 's, T> {
    type Target = T::Item<'w, 's>;

    fn deref(&self) -> &Self::Target {
        self.extra.deref()
    }
}

impl<'w, 's, T: SystemParam + 'static> DerefMut for State<'w, 's, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.extra.deref_mut()
    }
}

pub(crate) trait StateOps {
    fn get_entity_components(&self, archetype: &Archetype) -> impl Iterator<Item = ComponentId>;
    fn draw_node<'a>(
        &self,
        id: Entity,
        properties: impl Iterator<Item = (DynDebug<'a>, DebugName, bool, &'a ComponentInfo)>,
        buffer: &mut impl Write,
    ) -> std::io::Result<()>;
}
