#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use derive_more::Display;
use id_tree::{InsertBehavior, Node, NodeIdError, Tree};

use kodept_ast::graph::{GenericASTNode, GenericNodeId, PermTkn, SyntaxTree};
use kodept_ast::traits::Identifiable;
use kodept_inference::language::{var, Var};
use kodept_inference::r#type::PolymorphicType;
use kodept_macros::error::report::{ReportMessage, Severity};

use crate::scope::ScopeError::{Duplicate, NoScope};

#[derive(Display, Debug)]
pub enum ScopeError {
    #[display(fmt = "No scope available at this point")]
    NoScope,
    #[display(fmt = "Element with name `{_0}` already defined")]
    Duplicate(String),
}

#[derive(Default)]
pub struct ScopeTree {
    tree: Tree<Scope>,
    current_scope: Option<Id>,
}

pub struct Scope {
    start_from_id: GenericNodeId,
    name: Option<String>,
    types: HashMap<String, PolymorphicType>,
    variables: HashMap<String, Var>,
}

#[derive(Clone, Debug)]
pub struct ScopeSearch<'a> {
    tree: &'a Tree<Scope>,
    current_pos: Id,
    exclusive: bool,
}

type Id = id_tree::NodeId;

impl ScopeTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope<N>(&mut self, from: &N, name: Option<impl Into<String>>)
    where
        N: Identifiable + Into<GenericASTNode>,
    {
        let behaviour = match &self.current_scope {
            None => InsertBehavior::AsRoot,
            Some(id) => InsertBehavior::UnderNode(id),
        };
        let next_id = self
            .tree
            .insert(
                Node::new(Scope::new(from.get_id().widen(), name.map(|it| it.into()))),
                behaviour,
            )
            .expect("Current scope corrupted");
        self.current_scope = Some(next_id);
    }

    pub fn current_mut(&mut self) -> Result<&mut Scope, ScopeError> {
        Ok(self
            .tree
            .get_mut(self.current_scope.as_ref().ok_or(NoScope)?)
            .expect("Node was added recently")
            .data_mut())
    }

    pub fn pop_scope(&mut self) -> Result<(), ScopeError> {
        let current = self.current_scope.as_ref().ok_or(NoScope)?;
        self.current_scope = self
            .tree
            .ancestor_ids(current)
            .expect("Current node corrupted")
            .next()
            .cloned();
        Ok(())
    }

    fn of_node<N>(&self, node: &N, ast: &SyntaxTree, token: &PermTkn) -> Result<Id, ScopeError>
    where
        N: Identifiable + Into<GenericASTNode>,
    {
        let parents = {
            let mut current = node.get_id().widen();
            let mut result = vec![current];
            while let Some(parent) = ast.parent_of(current, token) {
                result.push(parent.get_id());
                current = parent.get_id();
            }
            result
        };

        let root = self.tree.root_node_id().ok_or(NoScope)?;
        self.tree
            .traverse_post_order_ids(root)
            .expect("Root node corrupted")
            .find(|id| {
                self.tree
                    .get(id)
                    .is_ok_and(|it| parents.contains(&it.data().start_from_id))
            })
            .ok_or(NoScope)
    }

    pub fn lookup<N>(
        &self,
        node: &N,
        ast: &SyntaxTree,
        token: &PermTkn,
    ) -> Result<ScopeSearch, ScopeError>
    where
        N: Identifiable + Into<GenericASTNode>,
    {
        let start = self.of_node(node, ast, token)?;
        Ok(ScopeSearch {
            tree: &self.tree,
            current_pos: start,
            exclusive: false,
        })
    }
}

impl Scope {
    fn new(from: GenericNodeId, name: Option<String>) -> Self {
        Self {
            start_from_id: from,
            name,
            types: Default::default(),
            variables: Default::default(),
        }
    }

    pub fn insert_type(
        &mut self,
        name: impl Into<String> + Clone,
        ty: PolymorphicType,
    ) -> Result<(), ScopeError> {
        if self.types.insert(name.clone().into(), ty).is_some() {
            return Err(Duplicate(name.into()));
        }
        Ok(())
    }

    pub fn insert_var(&mut self, name: impl Into<String> + Clone) -> Result<(), ScopeError> {
        if self
            .variables
            .insert(name.clone().into(), var(name.clone()))
            .is_some()
        {
            return Err(Duplicate(name.into()));
        }
        Ok(())
    }

    pub fn starts_from(&self) -> GenericNodeId {
        self.start_from_id
    }

    fn lookup_var(&self, name: impl AsRef<str>) -> Option<Var> {
        self.variables.get(name.as_ref()).cloned()
    }

    fn lookup_type(&self, name: impl AsRef<str>) -> Option<PolymorphicType> {
        self.types.get(name.as_ref()).cloned()
    }
}

impl ScopeSearch<'_> {
    pub fn exclusive(self) -> Self {
        Self {
            exclusive: true,
            ..self
        }
    }

    fn bubble_up<T>(&self, f: impl Fn(&Scope) -> Option<T>) -> Result<Option<T>, NodeIdError> {
        let mut current_pos = &self.current_pos;
        loop {
            let scope = self.tree.get(current_pos)?;
            match f(scope.data()) {
                None => {
                    current_pos = match self.tree.ancestor_ids(current_pos)?.next() {
                        None => return Ok(None),
                        Some(parent_id) => parent_id,
                    }
                }
                Some(out) => return Ok(Some(out)),
            }
        }
    }

    pub fn var(&self, name: impl AsRef<str> + Clone) -> Option<Var> {
        if self.exclusive {
            let scope = self.tree.get(&self.current_pos).expect("Tree corrupted");
            return scope.data().lookup_var(name);
        } else {
            self.bubble_up(|scope| scope.lookup_var(name.clone()))
                .expect("Tree corrupted")
        }
    }

    pub fn ty(&self, name: impl AsRef<str> + Clone) -> Option<PolymorphicType> {
        if self.exclusive {
            let scope = self.tree.get(&self.current_pos).expect("Tree corrupted");
            return scope.data().lookup_type(name);
        } else {
            self.bubble_up(|scope| scope.lookup_type(name.clone()))
                .expect("Tree corrupted")
        }
    }
}

impl Debug for ScopeTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.tree.write_formatted(f)
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "[{:?}:{name}]", self.start_from_id)?;
        } else {
            write!(f, "[{:?}]", self.start_from_id)?;
        }
        if !self.types.is_empty() {
            write!(
                f,
                " {{{}}}",
                self.types
                    .keys()
                    .map(|it| it.as_ref())
                    .intersperse(", ")
                    .collect::<String>()
            )?;
        }
        if !self.variables.is_empty() {
            write!(
                f,
                " {{{}}}",
                self.variables
                    .keys()
                    .cloned()
                    .intersperse(", ".to_string())
                    .collect::<String>()
            )?;
        }
        Ok(())
    }
}

impl From<ScopeError> for ReportMessage {
    fn from(value: ScopeError) -> Self {
        match value {
            NoScope => Self::new(Severity::Bug, "SM001", value.to_string()),
            Duplicate(_) => Self::new(Severity::Error, "SM002", value.to_string()),
        }
    }
}
