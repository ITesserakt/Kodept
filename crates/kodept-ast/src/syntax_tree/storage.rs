use crate::interaction::Interaction;
use crate::prelude::{CodeHolder, FromSyntax};
use crate::properties::{Lexeme, Root};
use crate::resource::rlt::{SyntaxResolver, SyntaxVariant};
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
        Root: FromSyntax<kodept_rlt::prelude::File>,
    {
        let mut world = World::new();
        let mut syntax = SyntaxResolver::empty(start);
        let whole_part = Root::from_syntax(syntax.root(), source_code);
        let id = unsafe { syntax.link(std::mem::transmute(SyntaxVariant::from(syntax.root()))) };
        world.insert_resource(syntax);
        world.spawn((Root, whole_part)).insert(Lexeme(id));
        AST { world }
    }

    pub fn interact(&mut self) -> Interaction {
        Interaction::new(&mut self.world)
    }
}
