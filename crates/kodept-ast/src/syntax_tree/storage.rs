use crate::interaction::Interaction;
use crate::prelude::{CodeHolder, FromSyntax};
use crate::properties::Root;
use crate::resource::rlt::SyntaxResolver;
use crate::syntax_tree::builder::Pool;
use bevy_ecs::prelude::World;
use kodept_rlt::prelude::RLT;

#[derive(Debug)]
pub struct AST {
    pub(crate) world: World,
}

impl AST {
    #[allow(unsafe_code)]
    pub fn recursively_build<Root>(start: RLT, source_code: impl CodeHolder) -> Self
    where
        Root: FromSyntax<Syntax = kodept_rlt::prelude::File>,
    {
        let mut world = World::new();
        let syntax = SyntaxResolver::empty(start);
        let pool = Pool::new(&syntax, world.entities());
        let whole_part =
            Root::from_syntax(pool.syntax_root(), source_code, pool).with_property(Root);
        unsafe {
            pool.link_syntax(whole_part.id(), pool.syntax_root());
        }
        whole_part.consume(&mut world);
        world.insert_resource(syntax);
        AST { world }
    }

    pub fn interact(&mut self) -> Interaction {
        Interaction::new(&mut self.world)
    }
}
