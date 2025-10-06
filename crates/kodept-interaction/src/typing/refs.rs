use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::{Commands, Entity, Local, Populated, With, Without};
use kodept_ast_nodes::term::Ref;
use kodept_inference::r#type::{MonomorphicType, TVar};
use crate::symbol::Declaration;
use crate::typing::Typed;

pub(super) fn system(
    query: Populated<(Entity, &Declaration), (With<Ref>, Without<Typed>)>,
    mut commands: Commands,
    mut vars_map: Local<EntityHashMap<TVar>>
) {
    for (id, declaration) in query.iter() {
        let var = vars_map
            .entry(declaration.0)
            .or_insert_with(|| TVar::new());
        commands.entity(id).insert(Typed(MonomorphicType::Var(*var)));
    }
}