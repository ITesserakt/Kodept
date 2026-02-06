use bevy_ecs::schedule::IntoScheduleConfigs;
mod collect;
mod resolve;

use crate::per_file::symbols::collect::{CollectSymbols, collect_params_on};
use crate::per_file::symbols::resolve::resolve_values;
use bevy_ecs::component::ComponentIdFor;
use bevy_ecs::prelude::Name;
use kodept_ast::Str;
use kodept_ast::export::Component;
use kodept_ast::prelude::{Erase, NodeId};
use kodept_ast::properties::{NodeProperty, RequireProperty};
use kodept_ast::syntax_tree::children::Wrapper;
use kodept_ast_nodes::{
    AnonFunction, Declaration, Module, NormalizedBlock, ResolvedType, ResolvedTypeAnnotation,
    TypeAnnotation, TypeRef, UnresolvedType, UserFunction, UserType, ValueCtor,
};
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;

struct ComponentIdForMapper<'a>(PhantomData<&'a ()>);
impl<'a> Wrapper for ComponentIdForMapper<'a> {
    type Wrapped<T: Component> = ComponentIdFor<'a, T>;
}
struct RefMapper<'a>(PhantomData<&'a ()>);
impl<'a> Wrapper for RefMapper<'a> {
    type Wrapped<T: Component> = &'a T;
}

define_phase! {
    pub phase ReferenceResolutionPhase[ReferenceResolutionPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        let collect_set = (
            collect_params_on(|ctor: &ValueCtor<UnresolvedType>| &*ctor.params),
            collect_params_on(|ctor: &ValueCtor<ResolvedType>| &*ctor.params),
            collect_params_on(|ctor: &AnonFunction<TypeAnnotation>| &*ctor.params),
            collect_params_on(|ctor: &AnonFunction<ResolvedTypeAnnotation>| &*ctor.params),
            collect_params_on(|ctor: &UserFunction<TypeAnnotation>| &*ctor.params),
            collect_params_on(|ctor: &UserFunction<ResolvedTypeAnnotation>| &*ctor.params),
            NormalizedBlock::system,
            (
                <UserType as CollectSymbols<Declaration>>::system,
                <UserType as CollectSymbols<()>>::system
            ).chain(),
            Module::system
        );

        engine.add_systems((collect_set, resolve_values).chain());
    }
}

#[derive(Component, Default)]
pub struct SymbolTable {
    order: Vec<(SymbolKind, NodeId)>,
    names: HashMap<SymbolName, usize>,
}
impl NodeProperty for SymbolTable {}
impl RequireProperty<SymbolTable> for Module {}
impl RequireProperty<SymbolTable> for UserType {}
impl<T: TypeRef<false>> RequireProperty<SymbolTable> for UserFunction<T> {}
impl RequireProperty<SymbolTable> for NormalizedBlock {}
impl<T: TypeRef<false>> RequireProperty<SymbolTable> for AnonFunction<T> {}
impl<T: TypeRef<true>> RequireProperty<SymbolTable> for ValueCtor<T> {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
struct SymbolName(Str);

#[derive(Debug, Clone)]
enum SymbolKind {
    Type,
    Function,
    Constructor,
    Parameter(usize),
    Variable,
}

impl Borrow<Str> for SymbolName {
    fn borrow(&self) -> &Str {
        &self.0
    }
}

impl Borrow<str> for SymbolName {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl From<Str> for SymbolName {
    fn from(value: Str) -> Self {
        SymbolName(value)
    }
}

impl From<&Name> for SymbolName {
    fn from(value: &Name) -> Self {
        Self(Str::from(value.as_str().to_owned()))
    }
}

impl From<&Str> for SymbolName {
    fn from(value: &Str) -> Self {
        match value {
            Str::Borrowed(x) => Self(Str::Borrowed(x)),
            Str::Owned(x) => Self(Str::Owned(x.clone())),
        }
    }
}

impl From<String> for SymbolName {
    fn from(value: String) -> Self {
        Self(Str::Owned(value))
    }
}

impl Display for SymbolKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Type => write!(f, "type"),
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Constructor => write!(f, "constructor"),
            SymbolKind::Parameter(_) => write!(f, "parameter"),
            SymbolKind::Variable => write!(f, "variable"),
        }
    }
}

impl SymbolTable {
    fn insert(
        &mut self,
        name: impl Into<SymbolName>,
        kind: SymbolKind,
        id: impl Erase,
    ) -> Result<(), (&SymbolKind, NodeId)> {
        let index = self.order.len();
        self.order.push((kind, id.erase()));
        if let Some(registered_index) = self.names.insert(name.into(), index) {
            let value = &self.order[registered_index];
            return Err((&value.0, value.1));
        }
        Ok(())
    }

    fn lookup<Q>(&self, name: &Q) -> Option<(NodeId, &SymbolKind)>
    where
        SymbolName: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = *self.names.get(name)?;
        let (kind, id) = &self.order[index];
        Some((*id, kind))
    }
}

impl Debug for SymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("SymbolTable");
        for (k, &v) in &self.names {
            d.field(k.borrow(), &self.order[v]);
        }
        d.finish()
    }
}
