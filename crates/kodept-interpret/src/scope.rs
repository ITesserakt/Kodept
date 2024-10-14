#![allow(clippy::unwrap_used)]

use crate::symbol::SymbolV2;
use derive_more::Display;
use kodept_ast::graph::{AnyNodeId, Identifiable, NodeId, SyntaxTree};
use kodept_ast::interning::SharedStr;
use kodept_inference::r#type::PolymorphicType;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::iter;
use thiserror::Error;

#[derive(Debug, Display, Error)]
#[display("Cannot get outer scope for root one")]
pub struct ScopePeelError;

#[derive(Display, Debug, Error)]
pub enum ScopeError {
    #[display("No scope available at this point")]
    NoScope,
    #[display("Element with name `{_0}` already defined")]
    Duplicate(String),
}

type Index = usize;

pub struct ScopeBuilder {
    // Scopes cannot be removed, so no gaps expected
    container: Vec<ScopeV2>,
    root_scope: Index,
    current_scope: Index,
    last_pushed_index: Index
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScopeV2<Type = Option<PolymorphicType>> {
    parent: Option<Index>,
    start_from: AnyNodeId,
    pub name: Option<SharedStr>,
    symbols: BTreeSet<SymbolV2<Type>>
}

#[derive(Debug, Copy, Clone)]
pub struct ScopeSearcher<'a, Type = Option<PolymorphicType>> {
    buffer: &'a [ScopeV2<Type>],
    root_scope: Index
}

#[derive(Debug)]
pub struct ScopeSearcherMut<'a, Type = Option<PolymorphicType>> {
    buffer: &'a mut [ScopeV2<Type>],
    root_scope: Index
}

impl ScopeV2 {
    pub fn new(start_from: AnyNodeId) -> Self {
        Self {
            parent: None,
            symbols: BTreeSet::new(),
            name: None,
            start_from
        }
    }
    
    /// Inserts a new symbol and returns the old one if presented
    pub fn insert_symbol(&mut self, symbol: SymbolV2) -> Option<SymbolV2> {
        self.symbols.replace(symbol)
    }
}

impl<T> ScopeSearcher<'_, T> {
    /// Finds the last scope that wraps up given node([`id`]).
    pub fn get_enclosing_scope(&self, id: AnyNodeId, ast: &SyntaxTree) -> &ScopeV2<T> {
        // First, try to find scope by checking start_from with id
        if let Some(strict_match) = self.buffer.iter().find(|it| it.start_from == id) {
            return strict_match;
        }

        let parents = iter::successors(Some(id), |&it| {
            Some(ast.parent_of(it)?.get_id())
        }).collect::<Vec<_>>();

        todo!()
    }
}

impl ScopeBuilder {
    pub fn new() -> Self {
        let root = ScopeV2::new(AnyNodeId::Root);
        Self {
            container: vec![root],
            // the first element in vec is root
            root_scope: 0,
            // root scope is the current one
            current_scope: 0,
            last_pushed_index: 0,
        }
    }

    pub fn search(&self) -> ScopeSearcher {
        ScopeSearcher {
            buffer: &self.container,
            root_scope: self.root_scope,
        }
    }

    pub fn search_mut(&mut self) -> ScopeSearcherMut {
        ScopeSearcherMut {
            buffer: &mut self.container,
            root_scope: self.root_scope,
        }
    }
    
    pub fn current_scope(&self) -> &ScopeV2 {
        &self.container[self.current_scope]
    }
    
    pub fn current_scope_mut(&mut self) -> &mut ScopeV2 {
        &mut self.container[self.current_scope]
    }

    pub fn root_scope(&self) -> &ScopeV2 {
        &self.container[self.root_scope]
    }

    pub fn push_scope(&mut self, start_from: AnyNodeId) -> &mut ScopeV2 {
        let mut scope = ScopeV2::new(start_from);
        scope.parent = Some(self.current_scope);

        self.last_pushed_index += 1;
        self.container.push(scope);
        self.current_scope = self.last_pushed_index;
        &mut self.container[self.last_pushed_index]
    }

    pub fn peel_scope(&mut self) -> Result<(), ScopePeelError> {
        let current = &self.container[self.current_scope];
        let parent = current.parent.ok_or(ScopePeelError)?;
        self.current_scope = parent;
        Ok(())
    }
}
