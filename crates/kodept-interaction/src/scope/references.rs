use crate::scope::storage::Scope;
use crate::scope::ScopeMapping;
use crate::wrapper::InteractionWrapper;
use crate::{done, Interaction, Result};
use bevy_ecs::prelude::{Entity, Query, Res};
use kodept_ast_nodes::term::Ref;
use std::convert::Infallible;

pub struct ReferenceResolver;

impl Interaction for ReferenceResolver {
    type Error = Infallible;

    fn interaction() -> InteractionWrapper<Self::Error> {
        InteractionWrapper::wrap(Self::system)
    }
}

impl ReferenceResolver {
    fn resolve_ref_without_context(id: Entity, node: &Ref, mapping: Res<ScopeMapping>, scopes: &Query<&Scope>) -> bool {
        let (scope_id, scope) = mapping.enclosing_scope(id, scopes);
        
        true
    }
    
    fn system(
        query: Query<(Entity, &Ref)>,
        scopes: Query<&Scope>,
        scopes_mapping: Res<ScopeMapping>,
    ) -> Result<Infallible> {
        query.into_iter().for_each(|(entity, node)| {
            if !node.context.global && node.context.items.is_empty() {
                
            }
        });
        done()
    }
}
