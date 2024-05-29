use std::borrow::Cow;
use std::collections::HashSet;
use std::rc::Rc;

use derive_more::From;
use slotmap::SparseSecondaryMap;

use kodept_ast::BodyFnDecl;
use kodept_ast::graph::{ChangeSet, GenericNodeId, GenericNodeKey, PermTkn, SyntaxTree};
use kodept_ast::traits::Identifiable;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_core::structure::{Located, rlt};
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::language::{Language, Var};
use kodept_inference::r#type::PolymorphicType;
use kodept_inference::traits::EnvironmentProvider;
use kodept_macros::error::report::{ReportMessage, Severity};
use kodept_macros::Macro;
use kodept_macros::traits::Context;

use crate::convert_model::ModelConvertibleNode;
use crate::node_family::TypeRestrictedNode;
use crate::scope::{ScopeError, ScopeSearch, ScopeTree};
use crate::type_checker::InferError::Unknown;
use crate::Witness;

pub struct CannotInfer;

pub struct TypeInfo<'a> {
    name: &'a str,
    ty: &'a PolymorphicType,
}

impl From<TypeInfo<'_>> for ReportMessage {
    fn from(value: TypeInfo<'_>) -> Self {
        Self::new(
            Severity::Note,
            "TC001",
            format!(
                "Type of function `{}` inferred to: {}",
                value.name, value.ty
            ),
        )
    }
}

impl From<CannotInfer> for ReportMessage {
    fn from(_: CannotInfer) -> Self {
        Self::new(Severity::Warning, "TC002", "Cannot infer type".to_string())
    }
}

pub struct TypeChecker<'a> {
    pub(crate) symbols: &'a ScopeTree,
    models: SparseSecondaryMap<GenericNodeKey, Rc<Language>>,
    evidence: Witness,
}

struct RecursiveTypeChecker<'a> {
    search: ScopeSearch<'a>,
    token: &'a PermTkn,
    tree: &'a SyntaxTree,
    models: &'a mut SparseSecondaryMap<GenericNodeKey, Rc<Language>>,
    evidence: Witness,
}

#[derive(From, Debug)]
pub enum InferError {
    AlgorithmW(AlgorithmWError),
    Scope(ScopeError),
    Unknown,
}

impl EnvironmentProvider<GenericNodeKey> for RecursiveTypeChecker<'_> {
    // TODO: handle error properly
    fn get(&mut self, key: &GenericNodeKey) -> Option<Cow<PolymorphicType>> {
        let id: GenericNodeId = (*key).into();
        let node = self.tree.get(id, self.token).expect("Node not found");

        if let Some(node) = node.try_cast::<TypeRestrictedNode>() {
            let search = self
                .search
                .as_tree()
                .lookup(node, self.tree, self.token)
                .ok()?;
            match node.type_of(&search, self.tree, self.token) {
                Execution::Failed(_) => return None,
                Execution::Completed(x) => return Some(Cow::Owned(x.generalize(&HashSet::new()))),
                Execution::Skipped => {}
            };
        }

        let model = self
            .models
            .entry(*key)
            .expect("Node not found")
            .or_insert_with(|| {
                Rc::new(
                    node.try_cast::<ModelConvertibleNode>()
                        .and_then(|node| {
                            node.to_model(
                                self.search.as_tree(),
                                self.tree,
                                self.token,
                                self.evidence,
                            )
                            .ok()
                        })
                        .expect("Cannot build model for node"),
                )
            })
            .clone();
        match model.infer(self) {
            Ok(x) => Some(Cow::Owned(x)),
            Err(_) => None,
        }
    }
}

impl EnvironmentProvider<Var> for RecursiveTypeChecker<'_> {
    fn get(&mut self, key: &Var) -> Option<Cow<PolymorphicType>> {
        let id = self.search.id_of_var(&key.name)?;
        let key: GenericNodeKey = id.into();
        self.get(&key)
    }
}

impl<'a> TypeChecker<'a> {
    pub fn new(symbols: &'a ScopeTree, evidence: Witness) -> Self {
        Self {
            symbols,
            models: Default::default(),
            evidence,
        }
    }

    pub fn into_inner(self) -> SparseSecondaryMap<GenericNodeKey, Rc<Language>> {
        self.models
    }
}

impl From<InferError> for ReportMessage {
    fn from(value: InferError) -> Self {
        match value {
            InferError::AlgorithmW(x) => Self::new(Severity::Error, "TI001", x.to_string()),
            InferError::Scope(x) => x.into(),
            Unknown => Self::new(Severity::Bug, "TI004", "Bug in implementation".to_string()),
        }
    }
}

impl Macro for TypeChecker<'_> {
    type Error = InferError;
    type Node = BodyFnDecl;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();
        let Some(tree) = context.tree().upgrade() else {
            return Execution::Skipped;
        };
        if matches!(side, VisitSide::Leaf | VisitSide::Exiting) {
            let search = self.symbols.lookup(&*node, &*tree, node.token())?;
            let mut rec = RecursiveTypeChecker {
                search,
                token: node.token(),
                tree: &*tree,
                models: &mut self.models,
                evidence: self.evidence,
            };
            let fn_location = context
                .access(&*node)
                .map_or(vec![], |it: &rlt::BodiedFunction| vec![it.id.location()]);
            let key: GenericNodeKey = node.get_id().widen().into();
            match rec.get(&key) {
                Some(ty) => {
                    context.add_report(
                        fn_location,
                        TypeInfo {
                            name: &node.name,
                            ty: &ty,
                        },
                    );
                }
                None => context.add_report(fn_location, CannotInfer),
            }
        }

        Execution::Completed(ChangeSet::new())
    }
}
