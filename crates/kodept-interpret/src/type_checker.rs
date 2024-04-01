use std::collections::HashMap;
use std::rc::Rc;

use kodept_ast::{BlockLevel, BodiedFunctionDeclaration, Body, Expression, Identifier, Operation};
use kodept_ast::graph::{ChangeSet, GhostToken, SyntaxTree};
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_core::structure::Located;
use kodept_core::structure::rlt::BodiedFunction;
use kodept_inference::algorithm_u::AlgorithmUError;
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::assumption::Assumptions;
use kodept_inference::Environment;
use kodept_inference::language::{app, lambda, Language, var};
use kodept_inference::language::Literal::Tuple;
use kodept_macros::error::report::{ReportMessage, Severity};
use kodept_macros::Macro;
use kodept_macros::traits::{Context, UnrecoverableError};

use crate::scope::SymbolTable;
use crate::symbol::TypeSymbol;

pub struct TypeChecker {
    _symbols: SymbolTable,
    _env: Environment,
    constants: HashMap<Rc<TypeSymbol>, usize>,
}

impl TypeChecker {
    fn populate_env(&mut self) {}

    pub fn new(symbol_table: SymbolTable) -> Self {
        let mut this = Self {
            _symbols: symbol_table,
            _env: Environment::default(),
            constants: Default::default(),
        };
        this.populate_env();
        this
    }
}

#[derive(Debug)]
struct CannotConvert;

#[cfg(debug_assertions)]
impl CannotConvert {
    fn new() -> Self {
        panic!("Cannot convert");
        CannotConvert
    }
}

#[derive(Copy, Clone)]
struct ConversionContext<'a> {
    checker: &'a TypeChecker,
    tree: &'a SyntaxTree,
    token: &'a GhostToken,
}

trait ConvertibleToModel<'a, Node>
where
    Self: 'a,
{
    fn convert(self, node: &'a Node) -> Result<Language, CannotConvert>;
}

impl<'a> ConvertibleToModel<'a, BodiedFunctionDeclaration> for ConversionContext<'a> {
    fn convert(self, node: &'a BodiedFunctionDeclaration) -> Result<Language, CannotConvert> {
        let expr = self.convert(node.body(self.tree, self.token))?;

        Ok(node
            .parameters(self.tree, self.token)
            .into_iter()
            .filter_map(|it| self.checker._symbols.lookup_by_node(it).ok())
            .map(|it| var(it))
            .fold(expr, |acc, next| lambda(next, acc).into()))
    }
}

impl<'a> ConvertibleToModel<'a, Body> for ConversionContext<'a> {
    fn convert(self, node: &'a Body) -> Result<Language, CannotConvert> {
        return if let Some(node) = node.as_block() {
            for item in node.items(self.tree, self.token) {
                // fold variables declarations
            }
            Ok(Tuple(vec![]).into())
        } else if let Some(node) = node.as_simple() {
            self.convert(node)
        } else {
            Err(CannotConvert::new())
        };
    }
}

impl<'a> ConvertibleToModel<'a, BlockLevel> for ConversionContext<'a> {
    fn convert(self, node: &'a BlockLevel) -> Result<Language, CannotConvert> {
        return if let Some(node) = node.as_func() {
            self.convert(node)
        } else if let Some(node) = node.as_init_var() {
            // self.convert(node)
            Err(CannotConvert::new())
        } else if let Some(node) = node.as_operation() {
            self.convert(node)
        } else {
            Err(CannotConvert::new())
        };
    }
}

impl<'a> ConvertibleToModel<'a, Operation> for ConversionContext<'a> {
    fn convert(self, node: &'a Operation) -> Result<Language, CannotConvert> {
        return if let Some(node) = node.as_application() {
            let func = self.convert(node.expr(self.tree, self.token))?;
            let params = node
                .params(self.tree, self.token)
                .into_iter()
                .map(|it: &Operation| self.convert(it))
                .try_fold(func, |acc, next| Ok(app(next?, acc).into()))?;
            Ok(params)
        } else if let Some(node) = node.as_expression() {
            self.convert(node)
        } else {
            Err(CannotConvert::new())
        };
    }
}

impl<'a> ConvertibleToModel<'a, Expression> for ConversionContext<'a> {
    fn convert(self, node: &'a Expression) -> Result<Language, CannotConvert> {
        return if let Some(node) = node.as_term().and_then(|it| it.as_reference()) {
            match &node.ident {
                Identifier::TypeReference { name } => todo!(),
                Identifier::Reference { name } => self
                    .checker
                    ._symbols
                    .lookup_var(name, false)
                    .map_err(|_| CannotConvert)
                    .map(|it| todo!()),
            }
        } else {
            Ok(Tuple(vec![]).into())
        };
    }
}

struct InferError(AlgorithmWError);

impl From<InferError> for ReportMessage {
    fn from(value: InferError) -> Self {
        match value.0 {
            AlgorithmWError::AlgorithmU(AlgorithmUError::CannotUnify { from, to }) => Self::new(
                Severity::Error,
                "TI002",
                format!("Expected to have type `{from}`, but have type `{to}``"),
            ),
            AlgorithmWError::UnknownVar(name) => {
                Self::new(Severity::Bug, "TI001", format!("`{name}` is not defined"))
            }
            AlgorithmWError::AlgorithmU(AlgorithmUError::InfiniteType { type_var, with }) => {
                Self::new(
                    Severity::Error,
                    "TI003",
                    format!("Infinite type detected: `{type_var}` ~ `{with}`"),
                )
            }
        }
    }
}

impl Macro for TypeChecker {
    type Error = UnrecoverableError;
    type Node = BodiedFunctionDeclaration;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();
        if !matches!(side, VisitSide::Entering | VisitSide::Leaf) {
            Execution::Skipped?
        }
        let Some(tree) = context.tree().upgrade() else {
            Execution::Skipped?
        };

        let ctx = ConversionContext {
            checker: self,
            tree: &tree,
            token: node.token(),
        };

        let mut a0 = Assumptions::empty();
        let Ok(model) = ctx.convert(&*node) else {
            return Execution::Skipped;
        };

        println!("{}", model);
        println!(
            "{}",
            match model.infer_with_env(&mut a0, &mut self._env) {
                Ok(x) => x,
                Err(e) => context.report_and_fail(
                    context
                        .access::<BodiedFunction>(&*node)
                        .map_or(vec![], |it| vec![it.id.location()]),
                    InferError(e)
                )?,
            }
        );

        Execution::Completed(ChangeSet::new())
    }
}
