#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use derive_more::Display;
use id_tree::{InsertBehavior, Node, Tree};

use kodept_ast::graph::GenericASTNode;
use kodept_ast::traits::Identifiable;
use kodept_inference::language::{var, Var};
use kodept_inference::r#type::PolymorphicType;
use kodept_macros::error::report::{ReportMessage, Severity};

use crate::scope::ScopeError::{Duplicate, NoScope};

#[derive(Display)]
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
    start_from_id: NodeId,
    name: Option<String>,
    types: HashMap<String, PolymorphicType>,
    variables: HashMap<String, Var>,
}

type Id = id_tree::NodeId;
type NodeId = kodept_ast::graph::NodeId<GenericASTNode>;

impl ScopeTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope<N>(&mut self, from: &N, name: Option<impl Into<String>>)
    where
        GenericASTNode: TryFrom<N>,
        N: Identifiable,
    {
        let behaviour = match &self.current_scope {
            None => InsertBehavior::AsRoot,
            Some(id) => InsertBehavior::UnderNode(id),
        };
        let next_id = self
            .tree
            .insert(
                Node::new(Scope::new(from.get_id().cast(), name.map(|it| it.into()))),
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
}

impl Scope {
    fn new(from: NodeId, name: Option<String>) -> Self {
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

    pub fn starts_from(&self) -> NodeId {
        self.start_from_id
    }
}

impl Debug for ScopeTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.tree.write_formatted(f)
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.start_from_id)?;
        if let Some(name) = &self.name {
            write!(f, " [{name}]")?;
        }
        if !self.types.is_empty() {
            write!(
                f,
                " {{{}}}",
                self.types
                    .iter()
                    .map(|(name, ty)| format!("{name} => {ty}"))
                    .intersperse(", ".to_string())
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
