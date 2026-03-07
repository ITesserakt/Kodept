use kodept_ecs::component::{ComponentId, ComponentIdFor};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::system::SystemParam;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

#[derive(SystemParam)]
pub(crate) struct NonVerboseComponents<'w, 's> {
    name: ComponentIdFor<'s, kodept_ast::properties::Name>,
    span: ComponentIdFor<'s, kodept_ast::properties::SourceSpan>,
    node: ComponentIdFor<'s, kodept_ast::properties::Node>,
    _phantom: PhantomData<&'w ()>,
}

impl NonVerboseComponents<'_, '_> {
    pub(crate) fn get(&self) -> [ComponentId; 3] {
        [self.node.get(), self.span.get(), self.name.get()]
    }
}

#[repr(transparent)]
pub(crate) struct DebugAsDisplay<T>(pub T);

impl<T: Debug> Display for DebugAsDisplay<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T: Debug> Debug for DebugAsDisplay<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
