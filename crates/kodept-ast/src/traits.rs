use crate::Str;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Component;
use kodept_core::structure::span::CodeHolder as BasicCodeHolder;

pub trait CodeHolder: BasicCodeHolder<Str = Str> {}
impl<T: BasicCodeHolder<Str = Str>> CodeHolder for T {}

pub trait FromSyntax<Syntax>: Sized {
    type Bundle: Bundle;
    type Error: Send + 'static;

    fn from_syntax(node: &Syntax, source: impl CodeHolder) -> Result<Self::Bundle, Self::Error>;
}

pub trait ASTNode: Component {}
