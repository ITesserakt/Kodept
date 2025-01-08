use crate::prelude::{ASTNode, CodeHolder, FromSyntax, NodeId};
use crate::properties::Node;
use crate::resource::rlt::SyntaxResolver;
use crate::syntax_tree::builder::Pool;
use bevy_ecs::prelude::{Component, World};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::RLT;
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
        for<'r> Root: FromSyntax<Syntax = rlt::File> + 'static,
    {
        let mut world = World::new();
        let syntax = SyntaxResolver::empty(start);
        let pool = Pool::new(syntax, &world);
        let whole_part = Root::from_syntax(pool.syntax_root(), source_code, &pool);
        let syntax = pool.into_syntax_resolver();
        whole_part.consume(&mut world);

        (Self { world }, syntax)
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
