use std::collections::VecDeque;
use crate::new_types::Symbol;
use crate::prelude::Reference;

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
        let mut refs = VecDeque::new();
        let mut current = self;
        loop {
            match current {
                Context::Global { .. } => return (Some(StartsFromRoot), Vec::from(refs)),
                Context::Local => return (None, Vec::from(refs)),
                Context::Inner { needle, parent } => {
                    refs.push_front(needle);
                    current = *parent;
                }
            }
        }
    }
}