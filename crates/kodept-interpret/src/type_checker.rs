use derive_more::From;
use nonempty_collections::{nev, IteratorExt, NEVec, NonEmptyIterator};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashSet;
use std::num::NonZeroU16;
use std::rc::Rc;

use kodept_ast::graph::{AnyNodeD, ChangeSet, GenericNodeId, GenericNodeKey, PermTkn, SyntaxTree};
use kodept_ast::traits::Identifiable;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_ast::BodyFnDecl;
use kodept_inference::algorithm_w::{AlgorithmWError, CompoundInferError};
use kodept_inference::language::{Language, Var};
use kodept_inference::r#type::PolymorphicType;
use kodept_inference::traits::EnvironmentProvider;
use kodept_macros::error::report::{ReportMessage, Severity};
use kodept_macros::Macro;
use RecursiveTypeCheckingError::{AlgoWError, MutuallyRecursive, NodeNotFound};

use crate::convert_model::ModelConvertibleNode;
use crate::node_family::TypeRestrictedNode;
use crate::scope::{ScopeError, ScopeSearch, ScopeTree};
use crate::type_checker::InferError::Unknown;
use crate::type_checker::RecursiveTypeCheckingError::InconvertibleToModel;
use crate::{Cache, Witness};

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
    models: Cache<Rc<Language>>,
    evidence: Witness,
    recursion_depth: NonZeroU16,
}

struct RecursiveTypeChecker<'a> {
    search: ScopeSearch<'a>,
    token: &'a PermTkn,
    tree: &'a SyntaxTree,
    models: &'a Cache<Rc<Language>>,
    evidence: Witness,
    current_recursion_depth: Cell<u16>,
}

#[derive(From, Debug)]
pub enum InferError {
    AlgorithmW(AlgorithmWError),
    Scope(ScopeError),
    Unknown,
}

#[derive(Debug, From)]
enum RecursiveTypeCheckingError {
    NodeNotFound(GenericNodeId),
    InconvertibleToModel(AnyNodeD),
    MutuallyRecursive,
    #[from]
    ScopeError(ScopeError),
    #[from]
    AlgoWError(AlgorithmWError),
}

#[derive(From, Debug)]
struct RecursiveTypeCheckingErrors {
    errors: NEVec<RecursiveTypeCheckingError>,
}

impl From<RecursiveTypeCheckingError> for RecursiveTypeCheckingErrors {
    fn from(value: RecursiveTypeCheckingError) -> Self {
        Self {
            errors: nev![value],
        }
    }
}

impl From<ScopeError> for RecursiveTypeCheckingErrors {
    fn from(value: ScopeError) -> Self {
        Self {
            errors: nev![value.into()],
        }
    }
}

impl From<InferError> for RecursiveTypeCheckingErrors {
    fn from(value: InferError) -> Self {
        Self {
            errors: match value {
                InferError::AlgorithmW(e) => nev![e.into()],
                InferError::Scope(e) => nev![e.into()],
                Unknown => panic!("Unknown error happened"),
            },
        }
    }
}

impl From<AlgorithmWError> for RecursiveTypeCheckingErrors {
    fn from(value: AlgorithmWError) -> Self {
        Self {
            errors: nev![value.into()],
        }
    }
}

impl From<RecursiveTypeCheckingError> for ReportMessage {
    fn from(value: RecursiveTypeCheckingError) -> Self {
        match value {
            NodeNotFound(id) => Self::new(
                Severity::Bug,
                "TC005",
                format!("Cannot find node with given id: {id}"),
            ),
            InconvertibleToModel(desc) => Self::new(
                Severity::Bug,
                "TC006",
                format!("Cannot convert node with description `{desc}` to model"),
            ),
            RecursiveTypeCheckingError::ScopeError(e) => e.into(),
            AlgoWError(e) => InferError::from(e).into(),
            MutuallyRecursive => Self::new(
                Severity::Error,
                "TC007",
                "Cannot type check due to mutual recursion".to_string(),
            )
            .with_notes(vec![
                "Adjust `recursion_depth` CLI option if needed".to_string()
            ]),
        }
    }
}

fn flatten(
    value: CompoundInferError<RecursiveTypeCheckingErrors>,
) -> NEVec<RecursiveTypeCheckingError> {
    match value {
        CompoundInferError::AlgoW(e) => nev![e.into()],
        CompoundInferError::Both(e, es) => {
            let errors: Vec<_> = es.into_iter().flat_map(|it| it.errors).collect();
            if let Some(mut errors) = NEVec::from_vec(errors) {
                errors.push(e.into());
                errors
            } else {
                nev![e.into()]
            }
        }
        CompoundInferError::Foreign(es) => es
            .into_iter()
            .flat_map(|it| it.errors)
            .to_nonempty_iter()
            .unwrap()
            .collect(),
    }
}

impl RecursiveTypeCheckingErrors {
    fn into_report_messages(self) -> Vec<ReportMessage> {
        self.errors.into_iter().map(ReportMessage::from).collect()
    }
}

impl EnvironmentProvider<GenericNodeKey> for RecursiveTypeChecker<'_> {
    type Error = RecursiveTypeCheckingErrors;

    fn maybe_get(&self, key: &GenericNodeKey) -> Result<Option<Cow<PolymorphicType>>, Self::Error> {
        let id: GenericNodeId = (*key).into();
        let node = self.tree.get(id, self.token).ok_or(NodeNotFound(id))?;

        if let Some(node) = node.try_cast::<TypeRestrictedNode>() {
            let search = self.search.as_tree().lookup(node, self.tree, self.token)?;
            match node.type_of(&search, self.tree, self.token) {
                Execution::Failed(e) => return Err(e.into()),
                Execution::Completed(x) => {
                    return Ok(Some(Cow::Owned(x.generalize(&HashSet::new()))))
                }
                Execution::Skipped => {}
            };
        }

        let depth = self.current_recursion_depth.get();
        match depth.checked_sub(1) {
            None => return Err(MutuallyRecursive.into()),
            Some(x) => self.current_recursion_depth.set(x)
        }
        
        let model = match self.models.get(*key) {
            Some(x) => x.clone(),
            None => {
                let model = node
                    .try_cast::<ModelConvertibleNode>()
                    .ok_or(InconvertibleToModel(node.describe()))?
                    .to_model(self.search.as_tree(), self.tree, self.token, self.evidence)?;
                let model = Rc::new(model);
                self.models.insert(*key, model.clone());
                model
            }
        };

        match model.infer(self) {
            Ok(x) => Ok(Some(Cow::Owned(x))),
            Err(e) => Err(RecursiveTypeCheckingErrors { errors: flatten(e) }),
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
    pub fn new(symbols: &'a ScopeTree, recursion_depth: NonZeroU16, evidence: Witness) -> Self {
        Self {
            symbols,
            models: Default::default(),
            evidence,
            recursion_depth,
        }
    }

    pub fn into_inner(self) -> Cache<Rc<Language>> {
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
        if !matches!(side, VisitSide::Leaf | VisitSide::Exiting) {
            return Execution::Skipped;
        }

        let search = self.symbols.lookup(&*node, &tree, node.token())?;
        let rec = RecursiveTypeChecker {
            search,
            token: node.token(),
            tree: &tree,
            models: &self.models,
            evidence: self.evidence,
            current_recursion_depth: Cell::new(self.recursion_depth.get()),
        };
        let fn_location = vec![];
        let key: GenericNodeKey = node.get_id().as_key().unwrap();
        match rec.maybe_get(&key) {
            Ok(Some(ty)) => {
                context.add_report(
                    fn_location,
                    TypeInfo {
                        name: &node.name,
                        ty: &ty,
                    },
                );
            }
            Ok(None) => context.add_report(fn_location.clone(), CannotInfer),
            Err(e) => e
                .into_report_messages()
                .into_iter()
                .for_each(|it| context.add_report(fn_location.clone(), it)),
        }

        Execution::Completed(ChangeSet::new())
    }
}
