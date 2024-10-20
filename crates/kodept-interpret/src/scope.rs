#![allow(clippy::unwrap_used)]

use crate::symbol::SymbolV2;
use derive_more::Display;
use kodept_ast::graph::{AnyNodeId, Identifiable, SyntaxTree};
use kodept_ast::interning::SharedStr;
use kodept_ast::ReferenceContext;
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

#[derive(Debug)]
pub struct ScopeBuilder {
    // Scopes cannot be removed, so no gaps expected
    container: Vec<ScopeV2>,
    root_scope: Index,
    current_scope: Index,
    last_pushed_index: Index,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScopeV2<Type = Option<PolymorphicType>> {
    parent: Option<Index>,
    start_from: AnyNodeId,
    name: Option<SharedStr>,
    symbols: BTreeSet<SymbolV2<Type>>,
    is_anonymous: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct ScopeSearcher<'a, Type = Option<PolymorphicType>> {
    buffer: &'a [ScopeV2<Type>],
    root_scope: Index,
}

#[derive(Debug)]
pub struct ScopeSearcherMut<'a, Type = Option<PolymorphicType>> {
    buffer: &'a mut [ScopeV2<Type>],
    root_scope: Index,
}

impl ScopeV2 {
    fn new(start_from: AnyNodeId) -> Self {
        Self {
            parent: None,
            symbols: BTreeSet::new(),
            name: None,
            start_from,
            is_anonymous: false,
        }
    }

    /// Inserts a new symbol and returns the old one if presented
    pub fn insert_symbol(&mut self, symbol: SymbolV2) -> Option<SymbolV2> {
        self.symbols.replace(symbol)
    }

    pub fn name(&self) -> Option<&str> {
        Some(self.name.as_ref()?.as_ref())
    }
}

impl<T> ScopeSearcher<'_, T> {
    /// Finds the nearest scope that wraps up given node([`id`]).
    pub fn get_enclosing_scope(&self, id: AnyNodeId, ast: &SyntaxTree) -> &ScopeV2<T> {
        // First, try to find scope by checking start_from with id
        if let Some(strict_match) = self.buffer.iter().find(|it| it.start_from == id) {
            return strict_match;
        }

        let parents =
            iter::successors(Some(id), |&it| Some(ast.parent_of(it)?.get_id())).collect::<Vec<_>>();

        todo!()
    }

    pub fn compute_context(&self, scope_index: Index) -> Option<ReferenceContext> {
        let scope = &self.buffer[scope_index];
        // Short circuit evaluation for the sake of optimization
        if scope.is_anonymous {
            return None;
        }

        let mut parents = vec![];
        let mut current = Some(scope_index);
        while let Some(parent) = current {
            let scope = &self.buffer[parent];
            if parent != self.root_scope {
                parents.push(scope.name.as_ref()?);
            }
            current = scope.parent;
        }

        Some(ReferenceContext::global(parents.into_iter().rev()))
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

    pub fn current_scope_index(&self) -> Index {
        self.current_scope
    }

    pub fn root_scope(&self) -> &ScopeV2 {
        &self.container[self.root_scope]
    }

    pub fn push_scope(
        &mut self,
        start_from: AnyNodeId,
        name: Option<&SharedStr>,
        anonymous: Option<bool>,
    ) -> &mut ScopeV2 {
        let mut scope = ScopeV2::new(start_from);
        scope.parent = Some(self.current_scope);
        scope.name = name.cloned();
        scope.is_anonymous = self.container[self.current_scope].is_anonymous
            || scope.name.is_none()
            || anonymous.unwrap_or(false);

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
