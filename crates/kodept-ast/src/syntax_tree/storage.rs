use crate::interaction::Interaction;
use crate::prelude::CodeHolder;
use crate::resource::rlt::SyntaxResolver;
use crate::traits::FromSyntax;
use bevy_ecs::prelude::World;
use derive_more::Constructor;
use kodept_core::file_name::FileDescriptor;
use kodept_rlt::prelude::{self as rlt, RLT};

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
    pub fn recursively_build_in<Root: FromSyntax<rlt::File>>(
        interaction: &mut Interaction,
        start: RLT,
        source_code: SourceCode<impl CodeHolder>,
    ) -> Result<(), Root::Error> {
        interaction.immediate_exclusive(|w| {
            let syntax = SyntaxResolver::build(start);

            let (root, root_id) = syntax.root();
            let whole_bundle = Root::from_syntax(root, source_code.code)?;
            w.insert_resource(syntax);
            let mut entity = w.spawn((
                whole_bundle,
                crate::properties::Root {
                    associated_file: source_code.descriptor,
                },
            ));
            entity.insert(crate::properties::Lexeme(root_id));

            Ok(())
        })
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
