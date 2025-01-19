use crate::prelude::{ASTNode, CodeHolder, FromSyntax, NodeId};
use crate::properties::Node;
use crate::resource::rlt::SyntaxResolver;
use crate::syntax_tree::builder::Pool;
use bevy_ecs::entity::Entities;
use bevy_ecs::prelude::{Component, Entity, World};
use bevy_ecs::world::CommandQueue;
use kodept_rlt::prelude::RLT;
use std::ops::Index;

#[derive(Debug)]
pub struct AST {
    world: World,
}

impl AST {
    pub fn recursively_build<Root>(
        start: RLT,
        source_code: impl CodeHolder,
    ) -> (Self, SyntaxResolver)
    where
        Root: FromSyntax<Syntax = kodept_rlt::prelude::File>,
    {
        let mut world = World::new();
        let syntax = Self::create_in_world::<Root>(start, source_code, &mut world);
        (AST { world }, syntax)
    }

    pub fn create_in_world<Root>(
        lexeme_tree: RLT,
        source_code: impl CodeHolder,
        world: &mut World,
    ) -> SyntaxResolver
    where
        Root: FromSyntax<Syntax = kodept_rlt::prelude::File>,
    {
        let mut resolver = SyntaxResolver::empty(lexeme_tree);
        let pool = Pool::new(&mut resolver, world.entities());
        let whole_part = Root::from_syntax(pool.syntax_root(), source_code, &pool);
        pool.link_syntax(whole_part.id(), pool.syntax_root());
        whole_part.consume(world);
        resolver
    }

    pub fn create_with_resolver<Root>(
        source_code: impl CodeHolder,
        resolver: &SyntaxResolver,
        entities: &Entities,
    ) -> (Entity, CommandQueue)
    where
        Root: FromSyntax<Syntax = kodept_rlt::prelude::File>,
    {
        let pool = Pool::new(resolver, entities);
        let whole_part = Root::from_syntax(pool.syntax_root(), source_code, &pool);
        pool.link_syntax(whole_part.id(), pool.syntax_root());
        (whole_part.id().as_inner(), whole_part.into_inner())
    }

    pub fn node_count(&self) -> usize {
        self.world
            .iter_entities()
            .filter(|it| it.contains::<Node>())
            .count()
    }

    pub fn contains<T: Component>(&self, id: NodeId) -> bool {
        self.world.entity(id.as_inner()).contains::<T>()
    }
}

impl<T> Index<NodeId<T>> for AST
where
    T: ASTNode,
{
    type Output = T;

    fn index(&self, index: NodeId<T>) -> &Self::Output {
        self.world
            .entity(index.as_inner())
            .get()
            .expect("Cannot get ast node")
    }
}
