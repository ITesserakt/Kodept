use crate::graph::nodes::PermTkn;

#[derive(Debug)]
pub struct FullAccess(pub PermTkn);

#[derive(Debug)]
pub struct ViewingAccess<'a>(pub &'a PermTkn);

#[derive(Debug)]
pub struct ModificationAccess<'a>(pub &'a mut PermTkn);

#[derive(Default, Debug)]
pub struct NoAccess;

pub trait CanAccess {
    fn tkn(&self) -> &PermTkn;
}

pub trait CanMutAccess: CanAccess {
    fn tkn_mut(&mut self) -> &mut PermTkn;
}

impl CanAccess for FullAccess {
    fn tkn(&self) -> &PermTkn {
        &self.0
    }
}

impl CanAccess for ViewingAccess<'_> {
    fn tkn(&self) -> &PermTkn {
        self.0
    }
}

impl CanAccess for ModificationAccess<'_> {
    fn tkn(&self) -> &PermTkn {
        self.0
    }
}

impl CanMutAccess for FullAccess {
    fn tkn_mut(&mut self) -> &mut PermTkn {
        &mut self.0
    }
}

impl CanMutAccess for ModificationAccess<'_> {
    fn tkn_mut(&mut self) -> &mut PermTkn {
        self.0
    }
}
