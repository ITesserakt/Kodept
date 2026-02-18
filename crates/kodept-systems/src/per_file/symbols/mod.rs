mod collect;
mod resolve;

use crate::per_file::symbols::collect::{CollectSymbols, check_module_names, collect_params_on};
use crate::per_file::symbols::resolve::{
    ResolvedTo, add_opaque_markers, add_passthrough_markers, ensure_absent,
    resolve_type_in_variables, resolve_types_in_anon_functions, resolve_types_in_foreign_functions,
    resolve_types_in_user_functions, resolve_types_in_value_ctors, resolve_values,
};
use crate::utils::LogSystemSetEx;
use kodept_ast::Str;
use kodept_ast::prelude::{Erase, NodeId};
use kodept_ast::properties::{Name, NodeProperty, RequireProperty};
use kodept_ast::syntax_tree::children::Wrapper;
use kodept_ast_nodes::{
    AnonFunction, Declaration, ForeignFunction, Module, NormalizedBlock, Param, ResolvedType,
    ResolvedTypeAnnotation, TypeAnnotation, TypeRef, UnresolvedType, UserFunction, UserType, Value,
    ValueCtor, Variable,
};
use kodept_ecs::component::{Component, ComponentIdFor};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::Without;
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
    pub phase ReferenceResolutionPhase[ReferenceResolutionPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        build(engine);
    }
}

fn build(engine: &mut PhaseEngine<ReferenceResolutionPhase>) {
    let setup_set = (add_passthrough_markers, add_opaque_markers);
    fn param_to_name<T>(param: &Param<T>) -> &Str {
        match param {
            Param::Positional { name, .. } => name,
            Param::Named { name, .. } => name,
        }
    }

    let collect_set = (
        (
            collect_params_on(
                |item: &ValueCtor<UnresolvedType>| &*item.params,
                param_to_name,
            ),
            collect_params_on(
                |item: &ValueCtor<ResolvedType>| &*item.params,
                param_to_name,
            ),
            collect_params_on(
                |item: &AnonFunction<TypeAnnotation>| &*item.params,
                |it| &it.name,
            ),
            collect_params_on(
                |item: &AnonFunction<ResolvedTypeAnnotation>| &*item.params,
                |it| &it.name,
            ),
            collect_params_on(
                |item: &UserFunction<TypeAnnotation>| &*item.params,
                param_to_name,
            ),
            collect_params_on(
                |item: &UserFunction<ResolvedTypeAnnotation>| &*item.params,
                param_to_name,
            ),
            NormalizedBlock::system,
            Module::system,
            check_module_names,
        )
            .trace_completion(),
        (
            <UserType as CollectSymbols<Declaration>>::system,
            <UserType as CollectSymbols<()>>::system,
        )
            .trace_completion()
            .chain(),
    );
    let resolve_set = (
        resolve_values,
        resolve_type_in_variables,
        resolve_types_in_user_functions,
        resolve_types_in_value_ctors,
        resolve_types_in_anon_functions,
        resolve_types_in_foreign_functions,
    )
        .trace_completion();
    let ensure_set = (
        ensure_absent::<Value, Without<ResolvedTo>>,
        ensure_absent::<Variable<TypeAnnotation>, ()>,
        ensure_absent::<UserFunction<TypeAnnotation>, ()>,
        ensure_absent::<AnonFunction<TypeAnnotation>, ()>,
        ensure_absent::<ForeignFunction<UnresolvedType>, ()>,
        ensure_absent::<ValueCtor<UnresolvedType>, ()>,
    );

    engine.add_systems(((setup_set, collect_set), resolve_set, ensure_set).chain());

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
    debug_registry.register::<resolve::ResolvedTo>();
}

#[derive(Component, Default)]
struct SymbolTable {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SymbolKind {
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
