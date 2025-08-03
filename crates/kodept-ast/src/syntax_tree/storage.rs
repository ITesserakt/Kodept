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
    pub fn recursively_build_in<Root>(
        interaction: &mut Interaction,
        start: RLT,
        source_code: SourceCode<impl CodeHolder>,
    ) -> Result<(), Root::Error>
    where
        Root: FromSyntax<kodept_rlt::prelude::File>,
    {
        let mut syntax = SyntaxResolver::empty(start);
        let whole_part = Root::from_syntax(syntax.root(), source_code.code)?;
        #[expect(
            unsafe_code,
            reason = "`syntax.root()` belongs to the syntax tree and it's safe to link it"
        )]
        let id = unsafe { syntax.link(std::mem::transmute(SyntaxVariant::from(syntax.root()))) };
        interaction.immediate_exclusive(|w| {
            w.insert_resource(syntax);
            let mut entity = w.spawn((
                crate::properties::Root {
                    associated_file: source_code.descriptor,
                },
                whole_part,
            ));
            entity.insert(Lexeme(id));
        });
        Ok(())
    }

    pub fn recursively_build<Root>(
        start: RLT,
        source_code: SourceCode<impl CodeHolder>,
    ) -> Result<Self, Root::Error>
    where
        Root: FromSyntax<kodept_rlt::prelude::File>,
    {
        let mut this = AST {
            world: World::new(),
        };
        let mut interaction = this.interact();
        Self::recursively_build_in::<Root>(&mut interaction, start, source_code)?;
        Ok(this)
    }

    pub fn interact(&mut self) -> Interaction<'_> {
        Interaction::new(&mut self.world)
    }
}
