use crate::symbol::Declaration;
use crate::typing::Typed;
use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::{Local, Populated, With, Without};
use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};
use kodept_ast::prelude::Children;
use kodept_ast_nodes::types::{ProdTy, Ty};
use kodept_inference::r#type::{MonomorphicType, TConstant};

pub(super) fn ty_system(
    query: Populated<(Entity, &Declaration), (With<Ty>, Without<Typed>)>,
    mut commands: Commands,
    mut constants_map: Local<EntityHashMap<TConstant>>,
) {
    for (id, decl) in query.iter() {
        let constant = constants_map
            .entry(decl.0)
            .or_insert_with(|| TConstant::new());
        commands
            .entity(id)
            .insert(Typed(MonomorphicType::Constant(*constant)));
    }
}

pub(super) fn prod_ty_system(
    tuples: Populated<(Entity, &Children), (Without<Typed>, With<ProdTy>)>,
    prods: Query<&Typed, With<ProdTy>>,
    types: Populated<&Typed, With<Ty>>,
    mut commands: Commands,
) {
    'outer: for (id, children) in tuples {
        let mut resulting_ty = vec![];
        for child in children {
            let Ok(ty) = prods.get(child).or_else(|_| types.get(child)) else {
                continue 'outer;
            };
            resulting_ty.push(&ty.0);
        }
        commands
            .entity(id)
            .insert(Typed(MonomorphicType::tuple(resulting_ty)));
    }
}
