use crate::prelude::{ASTNode, NodeRef};
use crate::properties::{Name, Node, RequireProperty, SourceSpan};

impl<'a, T> NodeRef<'a, &'a T> {
    pub fn name(&self) -> &Name
    where
        T: RequireProperty<Name>,
    {
        self.property()
    }

    pub fn kind(&self) -> &'static str
    where
        T: ASTNode,
    {
        self.property::<Node>().kind
    }
    
    pub fn span(&self) -> SourceSpan
    where 
        T: ASTNode
    {
        *self.property::<SourceSpan>()
    }
}
