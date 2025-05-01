use crate::interaction::Interaction;
use crate::prelude::{CodeHolder, FromSyntax};
use crate::resource::rlt::SyntaxResolver;
use bevy_ecs::prelude::World;
use kodept_rlt::prelude::RLT;
use crate::properties::Root;

#[derive(Debug)]
pub struct AST {
    pub(crate) world: World,
}

impl AST {
    #[allow(unsafe_code)]
    pub fn recursively_build<Root>(start: RLT, source_code: impl CodeHolder) -> Self
    where
        Root: FromSyntax<kodept_rlt::prelude::File>,
    {
        let mut world = World::new();
        let syntax = SyntaxResolver::empty(start);
        let whole_part = Root::from_syntax(syntax.root(), source_code);
        world.insert_resource(syntax);
        let root_id = world.spawn((Root, whole_part)).id();
        let syntax = world.resource_mut::<SyntaxResolver>();
        unsafe { syntax.insert(root_id, syntax.root()) };
        AST { world }
    }

    pub fn interact(&mut self) -> Interaction {
        Interaction::new(&mut self.world)
    }
}
