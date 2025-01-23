use crate::interaction::Interaction;
use crate::prelude::{CodeHolder, FromSyntax, NodeId};
use crate::properties::{Node, Root};
use crate::resource::rlt::SyntaxResolver;
use crate::syntax_tree::builder::Pool;
use bevy_ecs::prelude::{Component, World};
use kodept_rlt::prelude::RLT;

#[derive(Debug)]
pub struct AST {
    pub(crate) world: World,
}

impl AST {
    pub fn recursively_build<Root>(start: RLT, source_code: impl CodeHolder) -> Self
    where
        Root: FromSyntax<Syntax = kodept_rlt::prelude::File>,
    {
        let mut world = World::new();
        let syntax = SyntaxResolver::empty(start);
        let pool = Pool::new(&syntax, world.entities());
        let whole_part =
            Root::from_syntax(pool.syntax_root(), source_code, &pool).with_property(Root);
        pool.link_syntax(whole_part.id(), pool.syntax_root());
        whole_part.consume(&mut world);
        world.insert_resource(syntax);
        AST { world }
    }

    pub fn node_count(&self) -> usize {
        self.world
            .iter_entities()
            .filter(|it| it.contains::<Node>())
            .count()
    }

    pub fn contains<T: Component>(&self, id: NodeId) -> bool {
        self.world.entity(id).contains::<T>()
    }

    pub fn syntax_mut(&mut self) -> &mut SyntaxResolver {
        self.world.resource_mut::<SyntaxResolver>().into_inner()
    }

    pub fn syntax(&self) -> &SyntaxResolver {
        self.world.resource()
    }

    pub fn interact(&mut self) -> Interaction {
        Interaction::new(&mut self.world)
    }
}
