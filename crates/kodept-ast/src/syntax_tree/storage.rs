use crate::interaction::Interaction;
use crate::prelude::{CodeHolder, FromSyntax};
use crate::properties::Lexeme;
use crate::resource::rlt::{SyntaxResolver, SyntaxVariant};
use bevy_ecs::prelude::World;
use derive_more::Constructor;
use kodept_core::file_name::FileDescriptor;
use kodept_rlt::prelude::RLT;

#[derive(Debug)]
pub struct AST {
    world: World,
}

#[derive(Debug, Constructor)]
pub struct SourceCode<S: CodeHolder> {
    code: S,
    descriptor: FileDescriptor,
}

impl AST {
    #[allow(unsafe_code)]
    pub fn recursively_build<Root>(
        start: RLT,
        source_code: SourceCode<impl CodeHolder>,
    ) -> Result<Self, Root::Error>
    where
        Root: FromSyntax<kodept_rlt::prelude::File>,
    {
        let mut world = World::new();
        let mut syntax = SyntaxResolver::empty(start);
        let whole_part = Root::from_syntax(syntax.root(), source_code.code)?;
        let id = unsafe { syntax.link(std::mem::transmute(SyntaxVariant::from(syntax.root()))) };
        world.insert_resource(syntax);
        let mut entity = world.spawn((
            crate::properties::Root {
                associated_file: source_code.descriptor,
            },
            whole_part,
        ));
        entity.insert(Lexeme(id));
        Ok(AST { world })
    }

    pub fn interact(&mut self) -> Interaction<'_> {
        Interaction::new(&mut self.world)
    }
}
