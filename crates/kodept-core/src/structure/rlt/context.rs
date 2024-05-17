use crate::structure::rlt::new_types::Symbol;
use crate::structure::rlt::Reference;

pub struct StartsFromRoot;

#[derive(Debug, Clone, PartialEq)]
pub enum Context {
    Global {
        colon: Symbol
    },
    Local,
    Inner {
        parent: Box<Context>,
        needle: Reference
    }
}

impl Context {
    pub fn is_global(&self) -> bool {
        let mut current = self;
        
        loop {
            match current {
                Context::Global { .. } => return true,
                Context::Local => return false,
                Context::Inner { parent, .. } => {
                    current = parent.as_ref();
                    continue;
                }
            }
        }
    }
    
    pub fn unfold(self) -> (Option<StartsFromRoot>, Vec<Reference>) {
        let mut refs = vec![];
        let mut current = self;
        loop {
            match current {
                Context::Global { .. } => return (Some(StartsFromRoot), refs),
                Context::Local => return (None, refs),
                Context::Inner { needle, parent } => {
                    refs.push(needle);
                    current = *parent;
                }
            }
        }
    }
}