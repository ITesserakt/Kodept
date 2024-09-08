use derive_more::Constructor;
use kodept_ast::graph::dfs::DetachedDfsIter;
use kodept_ast::graph::SyntaxTree;
use kodept_macros::context::Context;
use std::ops::{Deref, DerefMut};

pub trait TreeTraversal {
    type Access;
    
    fn detached_iter(&self) -> DetachedDfsIter;
    
    fn get_tree(&self) -> &SyntaxTree<Self::Access>;
}

#[derive(Constructor)]
pub struct MacroContext<C> {
    capabilities: C
}

impl<C> DerefMut for MacroContext<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.capabilities
    }
}

impl<C> Deref for MacroContext<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.capabilities
    }
}

impl<C> Context<C> for MacroContext<C> {
    fn enrich<R>(self, f: impl FnOnce(C) -> R) -> impl Context<R>
    {
        MacroContext {
            capabilities: f(self.capabilities)
        }
    }
}