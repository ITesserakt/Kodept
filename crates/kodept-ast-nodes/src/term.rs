use bevy_ecs::prelude::Component;
use kodept_ast::{derive_node, Str};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone)]
pub struct ReferenceContext {
    pub global: bool,
    pub items: Vec<Str>,
}

#[derive(Debug, PartialEq, Component)]
pub struct Ref {
    pub context: ReferenceContext,
    pub ident: Str,
}

derive_node!(Ref);

impl ReferenceContext {
    pub fn empty(global: bool) -> Self {
        Self {
            global,
            items: vec![],
        }
    }

    pub fn global(items: impl IntoIterator<Item: Into<Cow<'static, str>>>) -> Self {
        Self {
            global: true,
            items: items.into_iter().map(|it| Str::from(it.into())).collect(),
        }
    }

    pub fn local(items: impl IntoIterator<Item: Into<Cow<'static, str>>>) -> Self {
        Self {
            global: false,
            items: items.into_iter().map(|it| Str::from(it.into())).collect(),
        }
    }

    /// Means that this context has no items in it and it is local
    pub const fn is_empty_local_context(&self) -> bool {
        !self.global && self.items.is_empty()
    }

    /// Means that this context has no items in it and it is global
    pub const fn is_empty_global_context(&self) -> bool {
        self.global && self.items.is_empty()
    }
}
