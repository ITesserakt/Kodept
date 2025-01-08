#![allow(clippy::unwrap_used)]

use crate::symbol::{SymbolKind, SymbolV2};
use dashmap::DashMap;
use derive_more::Display;
use itertools::Itertools;
use kodept_ast::graph::{AnyNodeId, Identifiable, SyntaxTree};
use kodept_ast::interning::SharedStr;
use kodept_ast::ReferenceContext;
use kodept_inference::r#type::PolymorphicType;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Display, Error)]
#[display("Cannot get outer scope for root one")]
pub struct ScopePeelError;

/// Contains a map from each node to the nearest enclosing scope
#[derive(Debug)]
pub(crate) struct EnclosingScopeCache {
    // usize represents scope id
    map: DashMap<AnyNodeId, usize>,
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

/// Immutable tree of scopes
#[derive(Debug, Clone)]
pub struct ScopeTree<Type = Option<PolymorphicType>> {
    array: Arc<[ScopeV2<Type>]>,
    root_scope: Index,
    cache: Rc<EnclosingScopeCache>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScopeV2<Type = Option<PolymorphicType>> {
    parent: Option<Index>,
    start_from: AnyNodeId,
    name: Option<SharedStr>,
    symbols: BTreeSet<SymbolV2<Type>>,
    is_anonymous: bool,
}

#[derive(Debug, Clone)]
pub struct ScopeSearcher<'a, Type = Option<PolymorphicType>> {
    buffer: &'a [ScopeV2<Type>],
    root_scope: Index,
    cache: Rc<EnclosingScopeCache>,
}

#[derive(Debug)]
pub struct ScopeWalker<'a, Type = Option<PolymorphicType>> {
    current: Option<Index>,
    view: &'a [ScopeV2<Type>],
}

impl<T> ScopeV2<T> {
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
    pub fn insert_symbol(&mut self, symbol: SymbolV2<T>) -> Option<SymbolV2<T>> {
        self.symbols.replace(symbol)
    }

    pub fn name(&self) -> Option<&str> {
        Some(self.name.as_ref()?.as_ref())
    }

    pub fn contains_symbol(
        &self,
        symbol_name: &SharedStr,
        mut symbol_kind_f: impl FnMut(SymbolKind) -> bool,
    ) -> bool {
        self.symbols
            .iter()
            .any(|v| &v.ident == symbol_name && symbol_kind_f(v.kind))
    }

    pub fn is_anonymous(&self) -> bool {
        self.is_anonymous
    }
}

impl<T> ScopeSearcher<'_, T> {
    /// Finds the nearest scope that wraps up given node([`id`]).
    pub fn get_enclosing_scope(&self, id: AnyNodeId, ast: &SyntaxTree) -> &ScopeV2<T> {
        if let Some(cached) = self.cache.map.get(&id) {
            return &self.buffer[*cached.value()];
        }

        let mut current = id;
        loop {
            // Try to find scope by comparing start_from with id
            if let Some((idx, strict_match)) = self
                .buffer
                .iter()
                .find_position(|it| it.start_from == current)
            {
                self.cache.map.insert(id, idx);
                return strict_match;
            }

            match ast.parent_of(current).map(|it| it.get_id()) {
                // node is out of ast, fail miserably
                None => unreachable!("Node with given id do not contained in the ast!"),
                Some(x) => current = x,
            }
        }
    }

    // TODO: optimise
    fn children_of(&self, node: Index) -> impl Iterator<Item = (Index, &ScopeV2<T>)> {
        self.buffer
            .iter()
            .enumerate()
            .filter(move |(_, it)| it.parent.as_ref().is_some_and(|id| id == &node))
    }

    // TODO: optimise
    fn index_of(&self, scope: &ScopeV2<T>) -> Index
    where
        T: PartialEq,
    {
        self.buffer.iter().position(|it| it == scope).unwrap()
    }

    /// Returns the last scope for the given context, otherwise returns the last scope that still matches context and context segment that failed
    pub fn matches<'a>(
        &self,
        context: &'a ReferenceContext,
    ) -> Result<&ScopeV2<T>, (&ScopeV2<T>, Option<&'a SharedStr>)> {
        if context.global {
            let mut current = self.root_scope;
            for item in &context.items {
                let mut children = self.children_of(current);
                match children.find(|(_, it)| it.name.as_ref().is_some_and(|name| name == item)) {
                    Some((match_id, _)) => current = match_id,
                    None => return Err((&self.buffer[current], Some(item))),
                };
            }
            return Ok(&self.buffer[current]);
        }
        // TODO: implement for non-global contexts


        Err((&self.buffer[self.root_scope], context.items.first()))
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

    pub fn walk_bottom_up<'a>(&'a self, start: &'a ScopeV2<T>) -> ScopeWalker<'a, T>
    where
        T: PartialEq,
    {
        let current = self.index_of(start);
        ScopeWalker {
            current: Some(current),
            view: self.buffer,
        }
    }
}

impl<'a, T> Iterator for ScopeWalker<'a, T> {
    type Item = &'a ScopeV2<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = &self.view[self.current?];
        self.current = result.parent;

        Some(result)
    }
}

impl Default for ScopeBuilder {
    fn default() -> Self {
        Self::new()
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
            cache: Rc::new(EnclosingScopeCache {
                map: Default::default(),
            }),
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

    pub fn complete(self) -> ScopeTree {
        ScopeTree {
            array: Arc::from(self.container.into_boxed_slice()),
            root_scope: self.root_scope,
            cache: Rc::new(EnclosingScopeCache {
                map: Default::default(),
            }),
        }
    }
}

impl<T> ScopeTree<T> {
    pub fn search(&self) -> ScopeSearcher<T> {
        ScopeSearcher {
            buffer: self.array.as_ref(),
            root_scope: self.root_scope,
            cache: self.cache.clone(),
        }
    }
}
