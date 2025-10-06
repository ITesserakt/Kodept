use bevy_ecs::prelude::{With, Without};
use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};
use kodept_ast_nodes::types::Ty;
use kodept_inference::r#type::MonomorphicType;

use crate::typing::Typed;

pub(super) fn system(query: Query<Entity, (With<Ty>, Without<Typed>)>, mut commands: Commands) {
    for (id) in query.iter() {
        let ty = MonomorphicType::constant();
        commands.entity(id).insert(Typed(ty));
    }
}
