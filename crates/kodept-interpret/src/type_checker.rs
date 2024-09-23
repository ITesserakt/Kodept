use derive_more::From;
use kodept_ast::graph::{AnyNodeD, ChangeSet, GenericNodeId, GenericNodeKey, SyntaxTree};
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_ast::BodyFnDecl;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::{rlt, Located};
use kodept_inference::algorithm_w::{AlgorithmWError, CompoundInferError};
use kodept_inference::language::{Language, Var};
use kodept_inference::r#type::PolymorphicType;
use kodept_inference::traits::EnvironmentProvider;
use kodept_macros::context::Context;
use kodept_macros::error::report::{IntoSpannedReportMessage, Label, Severity};
use kodept_macros::error::traits::SpannedError;
use kodept_macros::error::Diagnostic;
use kodept_macros::execution::Execution;
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::Macro;
use nonempty_collections::{nev, IteratorExt, NEVec, NonEmptyIterator};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashSet;
use std::num::NonZeroU16;
use std::rc::Rc;
use thiserror::Error;
use RecursiveTypeCheckingError::{MutuallyRecursive, NodeNotFound};

use crate::convert_model::ModelConvertibleNode;
use crate::node_family::TypeRestrictedNode;
use crate::scope::{ScopeError, ScopeSearch, ScopeTree};
use crate::type_checker::RecursiveTypeCheckingError::InconvertibleToModel;
use crate::Cache;

pub struct CannotInfer {
    point: CodePoint,
}

pub struct TypeInfo<'a> {
    name: &'a str,
    ty: &'a PolymorphicType,
    point: CodePoint,
}

impl<'a> IntoSpannedReportMessage for TypeInfo<'a> {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Note)
            .with_message(format!(
                "Type of function `{}` inferred to: {}",
                self.name, self.ty
            ))
            .with_label(Label::primary("here", self.point))
    }
}

impl IntoSpannedReportMessage for CannotInfer {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Error)
            .with_message("Cannot infer type")
            .with_label(Label::primary("here", self.point))
    }
}

pub struct TypeChecker<'a> {
    pub(crate) symbols: &'a ScopeTree,
    models: Cache<Rc<Language>>,
    recursion_depth: NonZeroU16,
}

struct RecursiveTypeChecker<'a> {
    search: ScopeSearch<'a>,
    tree: &'a SyntaxTree,
    rlt: &'a RLTAccessor<'a>,
    models: &'a Cache<Rc<Language>>,
    current_recursion_depth: Cell<u16>,
}

#[derive(Debug, Error)]
pub enum InferError {
    #[error(transparent)]
    AlgorithmW(AlgorithmWError),
    #[error(transparent)]
    Scope(ScopeError),
}

#[derive(Debug, Error, From)]
enum RecursiveTypeCheckingError {
    #[error("Node with id `{0}` was not found")]
    NodeNotFound(GenericNodeId),
    #[error("Node like `{0}` cannot convert to inner model")]
    InconvertibleToModel(AnyNodeD),
    #[error("Cannot type check due to mutual recursion")]
    MutuallyRecursive,
    #[error(transparent)]
    ScopeError(ScopeError),
    #[error(transparent)]
    AlgoWError(AlgorithmWError),
}

#[derive(Debug)]
struct RecursiveTypeCheckingErrors {
    errors: NEVec<SpannedError<RecursiveTypeCheckingError>>,
}

impl From<SpannedError<RecursiveTypeCheckingError>> for RecursiveTypeCheckingErrors {
    fn from(value: SpannedError<RecursiveTypeCheckingError>) -> Self {
        Self {
            errors: nev![value],
        }
    }
}

impl From<InferError> for RecursiveTypeCheckingError {
    fn from(value: InferError) -> Self {
        match value {
            InferError::AlgorithmW(x) => Self::AlgoWError(x),
            InferError::Scope(x) => Self::ScopeError(x),
        }
    }
}

fn flatten(
    error: CompoundInferError<RecursiveTypeCheckingErrors>,
    location: CodePoint,
) -> RecursiveTypeCheckingErrors {
    match error {
        CompoundInferError::AlgoW(x) => SpannedError::new(x.into(), location).into(),
        CompoundInferError::Both(x, errors) => {
            let tail = errors.into_iter().flat_map(|it| it.errors).collect();
            let errors = NEVec::from((SpannedError::new(x.into(), location).into(), tail));
            RecursiveTypeCheckingErrors { errors }
        }
        CompoundInferError::Foreign(errors) => {
            let errors = errors
                .into_iter()
                .flat_map(|it| it.errors)
                .to_nonempty_iter()
                .unwrap()
                .collect();
            RecursiveTypeCheckingErrors { errors }
        }
    }
}

impl EnvironmentProvider<GenericNodeKey> for RecursiveTypeChecker<'_> {
    type Error = RecursiveTypeCheckingErrors;

    fn maybe_get(&self, key: &GenericNodeKey) -> Result<Option<Cow<PolymorphicType>>, Self::Error> {
        let id: GenericNodeId = (*key).into();
        let location = self.rlt.get_unknown(id).unwrap().location();
        let node = self
            .tree
            .get(id)
            .ok_or(SpannedError::new(NodeNotFound(id).into(), location))?;

        if let Some(node) = node.try_cast::<TypeRestrictedNode>() {
            let search = self
                .search
                .as_tree()
                .lookup(node, &self.tree)
                .map_err(|e| {
                    SpannedError::new(RecursiveTypeCheckingError::ScopeError(e), location)
                })?;
            match node.type_of(&search, &self.tree, &self.rlt) {
                Execution::Failed(e) => {
                    return Err(e
                        .map(|it| match it {
                            InferError::AlgorithmW(x) => RecursiveTypeCheckingError::AlgoWError(x),
                            InferError::Scope(x) => RecursiveTypeCheckingError::ScopeError(x),
                        })
                        .into())
                }
                Execution::Completed(x) => {
                    return Ok(Some(Cow::Owned(x.generalize(&HashSet::new()))))
                }
                Execution::Skipped => {}
            };
        }

        let depth = self.current_recursion_depth.get();
        match depth.checked_sub(1) {
            None => {
                return Err(SpannedError::new(MutuallyRecursive, location)
                    .with_note("Adjust `recursion-depth` parameter")
                    .into())
            }
            Some(x) => self.current_recursion_depth.set(x),
        }

        let model = match self.models.get(*key) {
            Some(x) => x.clone(),
            None => {
                let model = node
                    .try_cast::<ModelConvertibleNode>()
                    .ok_or(InconvertibleToModel(node.describe()))
                    .map_err(|e| SpannedError::new(e, location))?
                    .to_model(self.search.as_tree(), &self.tree, &self.rlt)
                    .map_err(|e| e.map(|it| it.into()))?;
                let model = Rc::new(model);
                self.models.insert(*key, model.clone());
                model
            }
        };

        match model.infer(self) {
            Ok(x) => Ok(Some(Cow::Owned(x))),
            Err(e) => Err(flatten(e, location)),
        }
    }
}

impl EnvironmentProvider<Var> for RecursiveTypeChecker<'_> {
    type Error = RecursiveTypeCheckingErrors;

    fn maybe_get(&self, key: &Var) -> Result<Option<Cow<PolymorphicType>>, Self::Error> {
        let Some(id) = self.search.id_of_var(&key.name) else {
            return Ok(None);
        };
        let key: GenericNodeKey = id.as_key().unwrap();
        self.maybe_get(&key)
    }
}

impl<'a> TypeChecker<'a> {
    pub fn new(symbols: &'a ScopeTree, recursion_depth: NonZeroU16) -> Self {
        Self {
            symbols,
            models: Default::default(),
            recursion_depth,
        }
    }

    pub fn into_inner(self) -> Cache<Rc<Language>> {
        self.models
    }
}

impl Macro for TypeChecker<'_> {
    type Error = InferError;
    type Node = BodyFnDecl;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Context<'_>,
    ) -> Execution<Self::Error, ChangeSet> {
        let node_id = guard.allow_last()?;
        let node = ctx.ast.get(node_id).unwrap();

        let search = self.symbols.lookup(node, &ctx.ast).unwrap();
        let key: GenericNodeKey = node_id.as_key().unwrap();
        let rec = RecursiveTypeChecker {
            search,
            tree: &ctx.ast,
            rlt: &ctx.rlt,
            models: &self.models,
            current_recursion_depth: Cell::new(self.recursion_depth.get()),
        };
        let ty = rec.maybe_get(&key);

        let fn_location = ctx
            .rlt
            .get(node_id)
            .map(|it: &rlt::BodiedFunction| it.id.location())
            .unwrap();

        match ty {
            Ok(Some(ty)) => {
                ctx.collector.report(
                    ctx.current_file.id,
                    TypeInfo {
                        name: &node.name,
                        ty: &ty,
                        point: fn_location,
                    },
                );
            }
            Ok(None) => ctx.report(CannotInfer { point: fn_location }),
            Err(e) => e.errors.into_iter().for_each(|e| ctx.report(e)),
        }

        Execution::Completed(ChangeSet::new())
    }
}
