use crate::prelude::{ASTNode, CodeHolder, FromSyntax, NodeId};
use crate::properties::Node;
use crate::resource::rlt::SyntaxResolver;
use crate::syntax_tree::builder::Pool;
use bevy_ecs::prelude::{Component, World};
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
    
    pub fn create_in_world<Root>(lexeme_tree: RLT, source_code: impl CodeHolder, world: &mut World) -> SyntaxResolver
    where 
        Root: FromSyntax<Syntax = kodept_rlt::prelude::File>
    {
        let resolver = SyntaxResolver::empty(lexeme_tree);
        let pool = Pool::new(resolver, world);
        let whole_part = Root::from_syntax(pool.syntax_root(), source_code, &pool);
        pool.link_syntax(whole_part.id(), pool.syntax_root());
        let syntax = pool.into_syntax_resolver();
        whole_part.consume(world);
        syntax
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
