#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use derive_more::Display;
use id_tree::{InsertBehavior, Node, Tree, TreeBuilder};

use crate::{Errors, Path};
use crate::Errors::{AlreadyDefined, UnresolvedReference};
use crate::symbol::{Symbol, TypeSymbol, VarSymbol};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Display)]
struct Tag(String);

#[derive(Hash, Eq, PartialEq, Debug)]
struct Name(String);

type Id = id_tree::NodeId;

pub struct SymbolTable {
    table: HashMap<Id, Symbol>,
    structure: Tree<Tag>,
    current_tag_id: Id,
}

impl SymbolTable {
    fn get_current_tag(&self) -> &Tag {
        self.structure
            .get(&self.current_tag_id)
            .expect("Cannot get current node")
            .data()
    }

    fn get_current_parent_id(&self) -> Option<&Id> {
        self.structure.get(&self.current_tag_id).ok()?.parent()
    }

    fn tag_chain(&self, tag: &Id) -> impl DoubleEndedIterator<Item = &Tag> {
        let vec: Vec<_> = self.structure.ancestors(tag).unwrap().collect();
        vec.into_iter().rev().map(|it| it.data())
    }

    pub fn lookup(&self, path: &Path, exclusive: bool) -> Result<Symbol, Errors> {
        let mut current_scope = Some(&self.current_tag_id);
        loop {
            if let Some(scope) = current_scope {
                let node = self.structure.get(scope).unwrap();
                let name = node.children().iter().find(|it| {
                    self.structure
                        .get(it)
                        .is_ok_and(|node| &node.data().0 == path)
                });
                match name.and_then(|it| self.table.get(it)) {
                    None if exclusive => return Err(UnresolvedReference(path.clone())),
                    None => current_scope = node.parent(),
                    Some(x) => return Ok(x.clone()),
                }
            } else {
                return Err(UnresolvedReference(path.clone()));
            }
        }
    }

    pub fn lookup_type(&self, path: &Path, exclusive: bool) -> Result<Rc<TypeSymbol>, Errors> {
        self.lookup(path, exclusive)?
            .try_into()
            .map_err(|_| UnresolvedReference(path.clone()))
    }

    pub fn lookup_var(&self, path: &Path, exclusive: bool) -> Result<Rc<VarSymbol>, Errors> {
        self.lookup(path, exclusive)?
            .try_into()
            .map_err(|_| UnresolvedReference(path.clone()))
    }

    pub fn insert<T: Clone + Into<Symbol>>(&mut self, symbol: T) -> Result<T, Errors> {
        let out = symbol;
        let symbol = out.clone().into();
        if self
            .structure
            .children(&self.current_tag_id)
            .unwrap()
            .any(|it| &it.data().0 == symbol.path())
        {
            return Err(AlreadyDefined(symbol.path().clone()));
        }
        let id = self
            .structure
            .insert(
                Node::new(Tag(symbol.path().clone())),
                InsertBehavior::UnderNode(&self.current_tag_id),
            )
            .unwrap();
        self.table.insert(id, symbol);
        Ok(out)
    }

    pub fn new(root: Path) -> Self {
        let structure = TreeBuilder::new().with_root(Node::new(Tag(root))).build();
        let current_tag_id = structure.root_node_id().unwrap().clone();
        Self {
            table: Default::default(),
            structure,
            current_tag_id,
        }
    }

    pub fn begin_scope(&mut self, name: Path) -> Result<(), Errors> {
        self.current_tag_id = match self
            .structure
            .children_ids(&self.current_tag_id)
            .unwrap()
            .find(|it| {
                self.structure
                    .get(it)
                    .is_ok_and(|node| node.data().0 == name)
            }) {
            None => self
                .structure
                .insert(
                    Node::new(Tag(name)),
                    InsertBehavior::UnderNode(&self.current_tag_id),
                )
                .unwrap(),
            Some(x) => x.clone(),
        };
        Ok(())
    }

    pub fn end_scope(&mut self, name: &Path) -> Result<(), Errors> {
        if &self.get_current_tag().0 != name {
            return Err(Errors::WrongScope {
                expected: name.clone(),
                found: self.get_current_tag().0.clone(),
            });
        }
        let parent = self.get_current_parent_id();
        if let Some(id) = parent {
            self.current_tag_id = id.clone();
        }
        Ok(())
    }
}

impl Debug for SymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        self.structure.write_formatted(f)?;
        write!(f, "SymbolTable {{")?;
        let table = self
            .table
            .iter()
            .fold(String::new(), |acc, (tag_id, symbol)| {
                let path = self
                    .tag_chain(tag_id)
                    .fold(String::new(), |acc, next_id| format!("{acc}::{}", next_id));
                let pointer = if tag_id == &self.current_tag_id {
                    ">   "
                } else {
                    "    "
                };
                format!("{acc}\n{pointer}{path} => {symbol:?}")
            });
        writeln!(f, "{table}")?;
        writeln!(f, "}}")?;
        Ok(())
    }
}
