mod collect;
mod resolve;

use crate::per_file::symbols::collect::{CollectSymbolsIn, check_module_names};
use crate::per_file::symbols::resolve::{
    ResolveTypeIn, ResolveValues, ResolvedTo, StrictType, add_opaque_markers,
    add_passthrough_markers, ensure_absent,
};
use crate::per_file::utils::IntoNodeSystem;
use kodept_ast::Str;
use kodept_ast::prelude::{Erase, NodeId};
use kodept_ast::properties::{Name, NodeProperty, RequireProperty};
use kodept_ast::syntax_tree::children::Wrapper;
use kodept_ast_nodes::{
    AnonFunction, ForeignFunction, Module, NamedParams, NormalizedBlock, Param, Params,
    TypeAnnotation, UnresolvedType, UserFunction, UserType, Value, ValueCtor, Variable,
};
use kodept_ecs::component::{Component, ComponentIdFor};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::{With, Without};
use kodept_ecs::schedule::IntoScheduleConfigs;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
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
    pub phase SymbolsPhase[SymbolsPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        build(engine);
    }
}

define_phase! {
    phase SetupReferenceResolutionPhase[SetupReferenceResolutionLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(add_passthrough_markers);
        engine.add_systems(add_opaque_markers);
    }
}

define_phase! {
    phase CollectSymbolsPhase[CollectSymbolsPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(CollectSymbolsIn::<ValueCtor, _, Param>::system_with_input(SymbolKind::Parameter));

        engine.add_systems((
            CollectSymbolsIn::<UserFunction, Params, Param>::system_with_input(SymbolKind::Parameter),
            CollectSymbolsIn::<UserFunction, NamedParams, Param>::system_with_input(SymbolKind::Parameter)
        ).chain());

        engine.add_systems(CollectSymbolsIn::<AnonFunction, _, Param>::system_with_input(SymbolKind::Parameter));

        engine.add_systems((
            CollectSymbolsIn::<NormalizedBlock, _, Variable>::system(),
            CollectSymbolsIn::<NormalizedBlock, _, UserFunction>::system_with_input(SymbolKind::Function),
        ).chain());

        engine.add_systems((
            CollectSymbolsIn::<Module, _, UserFunction>::system_with_input(SymbolKind::Function),
            CollectSymbolsIn::<Module, _, UserType>::system_with_input(SymbolKind::Type),
        ).chain());

        engine.add_systems((
            CollectSymbolsIn::<UserType, _, ValueCtor>::system(),
            CollectSymbolsIn::<UserType, _, UserFunction>::system_with_input(SymbolKind::Function)
        ).chain());

        engine.add_systems(check_module_names);
    }
}

define_phase! {
    phase ResolvePhase[ResolvePhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        let resolve_set = (
            ResolveValues::system(),
            ResolveTypeIn::<Variable>::system_with_input(()),
            ResolveTypeIn::<UserFunction>::system_with_input("a return type"),
            ResolveTypeIn::<AnonFunction>::system_with_input("a return type"),
            ResolveTypeIn::<ForeignFunction>::system_with_input((StrictType, "a return type")),
            ResolveTypeIn::<Param>::system_with_input("a parameter"),
            ResolveTypeIn::<Param>::system_with_input((StrictType, "a parameter")),
        );
        let ensure_set = (
            ensure_absent::<Value, Without<ResolvedTo>>,
            ensure_absent::<Variable, With<TypeAnnotation>>,
            ensure_absent::<UserFunction, With<TypeAnnotation>>,
            ensure_absent::<AnonFunction, With<TypeAnnotation>>,
            ensure_absent::<ForeignFunction, With<UnresolvedType>>,
            ensure_absent::<ValueCtor, With<UnresolvedType>>,
        );

      engine.add_systems((resolve_set, ensure_set).chain());
    }
}

fn build(engine: &mut PhaseEngine<SymbolsPhase>) {
    engine
        .install(SetupReferenceResolutionPhase)
        .install(CollectSymbolsPhase)
        .install(ResolvePhase);

    #[cfg(feature = "reflection")]
    engine.add_systems(register_reflection_info);
}

#[cfg(feature = "reflection")]
fn register_reflection_info(
    mut debug_registry: bevy_ecs::prelude::If<
        bevy_ecs::prelude::ResMut<kodept_ast::resource::reflection::DebugRegistry>,
    >,
) {
    debug_registry.register::<SymbolTable>();
    debug_registry.register::<ResolvedTo>();
}

#[derive(Component, Default)]
struct SymbolTable {
    order: Vec<(SymbolKind, NodeId)>,
    names: HashMap<SymbolName, usize>,
}
impl NodeProperty for SymbolTable {}
impl RequireProperty<SymbolTable> for Module {}
impl RequireProperty<SymbolTable> for UserType {}
impl RequireProperty<SymbolTable> for UserFunction {}
impl RequireProperty<SymbolTable> for NormalizedBlock {}
impl RequireProperty<SymbolTable> for AnonFunction {}
impl RequireProperty<SymbolTable> for ValueCtor {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
struct SymbolName(Str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SymbolKind {
    Type,
    Function,
    Constructor,
    Parameter,
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
            SymbolKind::Parameter => write!(f, "parameter"),
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
    ) -> Result<(), (SymbolKind, NodeId)> {
        match self.names.entry(name.into()) {
            Entry::Occupied(entry) => {
                let value = self.order[*entry.get()];
                Err((value.0, value.1))
            }
            Entry::Vacant(entry) => {
                let index = self.order.len();
                self.order.push((kind, id.erase()));
                entry.insert(index);
                Ok(())
            }
        }
    }

    fn lookup<Q>(&self, name: &Q) -> Option<(NodeId, SymbolKind)>
    where
        SymbolName: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = *self.names.get(name)?;
        let (kind, id) = &self.order[index];
        Some((*id, kind.clone()))
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
