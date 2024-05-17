use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;

use kodept_ast::graph::{PermTkn, SyntaxTree};
use kodept_ast::traits::Identifiable;
use kodept_ast::{
    Application, BlockLevel, BlockLevelEnum, BodiedFunctionDeclaration, Body, BodyEnum, Expression,
    ExpressionBlock, ExpressionEnum, Identifier, IfExpression, InitializedVariable, Lambda,
    Literal, LiteralEnum, Operation, OperationEnum, ParameterEnum, Reference, Term, TermEnum,
};
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::Literal::{Floating, Tuple};
use kodept_inference::language::{app, lambda, r#if, r#let, var, Language};
use kodept_inference::r#type::MonomorphicType;
use kodept_inference::Environment;
use Identifier::TypeReference;

use crate::node_family::{TypeDerivableNode, TypeDerivableNodeEnum};
use crate::scope::ScopeTree;
use crate::type_checker::{InferError, TypeChecker};

impl TypeDerivableNode {
    pub fn type_of<'a>(
        &self,
        ast: &'a SyntaxTree,
        token: &'a PermTkn,
        scopes: &'a ScopeTree,
        assumptions: &mut Assumptions,
        environment: &mut Environment,
    ) -> Result<(Rc<Language>, MonomorphicType), InferError> {
        let helper = ConversionHelper { scopes, ast, token };
        let model = Rc::new(helper.convert(self)?);
        let derived_type = Language::infer_with_env(model.clone(), assumptions, environment)?;
        Ok((model, derived_type))
    }
}

impl ToModelFrom<TypeDerivableNode> for ConversionHelper<'_> {
    fn convert(self, node: &TypeDerivableNode) -> Result<Language, InferError> {
        match node.as_enum() {
            TypeDerivableNodeEnum::Function(x) => self.convert(x),
            TypeDerivableNodeEnum::ExpressionBlock(x) => self.convert(x),
            TypeDerivableNodeEnum::InitVar(x) => self.convert(x),
            TypeDerivableNodeEnum::Lambda(x) => self.convert(x),
            TypeDerivableNodeEnum::Application(x) => self.convert(x),
            TypeDerivableNodeEnum::IfExpr(x) => self.convert(x),
            TypeDerivableNodeEnum::Reference(x) => self.convert(x),
            TypeDerivableNodeEnum::Literal(x) => self.convert(x),
        }
    }
}

impl TypeChecker {
    #[allow(private_bounds)]
    pub(crate) fn to_model<'a, N>(
        &'a self,
        ast: &'a SyntaxTree,
        token: &'a PermTkn,
        node: &N,
    ) -> Result<Language, InferError>
    where
        ConversionHelper<'a>: ToModelFrom<N>,
    {
        let helper = ConversionHelper {
            scopes: &self.symbols,
            ast,
            token,
        };
        helper.convert(node)
    }
}

#[derive(Copy, Clone)]
struct ConversionHelper<'a> {
    scopes: &'a ScopeTree,
    ast: &'a SyntaxTree,
    token: &'a PermTkn,
}

#[inline]
const fn unit() -> Language {
    Language::Literal(Tuple(vec![]))
}

trait ToModelFrom<N> {
    fn convert(self, node: &N) -> Result<Language, InferError>;
}

trait ExtractName {
    fn extract_name(&self, tree: &SyntaxTree, token: &PermTkn) -> Cow<str>;
}

impl ToModelFrom<Body> for ConversionHelper<'_> {
    fn convert(self, node: &Body) -> Result<Language, InferError> {
        match node.as_enum() {
            BodyEnum::Block(x) => self.convert(x),
            BodyEnum::Simple(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<BlockLevel> for ConversionHelper<'_> {
    fn convert(self, node: &BlockLevel) -> Result<Language, InferError> {
        match node.as_enum() {
            BlockLevelEnum::Func(x) => self.convert(x),
            BlockLevelEnum::InitVar(x) => self.convert(x),
            BlockLevelEnum::Operation(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<BodiedFunctionDeclaration> for ConversionHelper<'_> {
    fn convert(self, node: &BodiedFunctionDeclaration) -> Result<Language, InferError> {
        let expr = self.convert(node.body(self.ast, self.token))?;
        let scope = self.scopes.lookup(node, self.ast, self.token)?;
        let mut bindings = node
            .parameters(self.ast, self.token)
            .into_iter()
            .map(|it| {
                let name = match it.as_enum() {
                    ParameterEnum::Typed(x) => &x.name,
                    ParameterEnum::Untyped(x) => &x.name,
                };
                scope
                    .var(name)
                    .ok_or(AlgorithmWError::UnknownVar(var(name)))
            })
            .peekable();
        if bindings.peek().is_some() {
            bindings.try_rfold(expr, |acc, next| Ok(lambda(next?, acc).into()))
        } else {
            Ok(lambda(var("()"), expr).into())
        }
    }
}

impl ToModelFrom<Operation> for ConversionHelper<'_> {
    fn convert(self, node: &Operation) -> Result<Language, InferError> {
        match node.as_enum() {
            OperationEnum::Application(node) => self.convert(node),
            OperationEnum::Access(_) => todo!(),
            OperationEnum::Unary(_) => todo!(),
            OperationEnum::Binary(_) => todo!(),
            OperationEnum::Block(x) => self.convert(x),
            OperationEnum::Expression(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<Application> for ConversionHelper<'_> {
    fn convert(self, node: &Application) -> Result<Language, InferError> {
        let expr = self.convert(node.expr(self.ast, self.token))?;
        let mut params = node
            .params(self.ast, self.token)
            .into_iter()
            .rev()
            .map(|it| self.convert(it))
            .peekable();
        if params.peek().is_some() {
            params.try_fold(expr, |acc, next| Ok(app(next?, acc).into()))
        } else {
            Ok(app(unit(), expr).into())
        }
    }
}

impl ToModelFrom<ExpressionBlock> for ConversionHelper<'_> {
    fn convert(self, node: &ExpressionBlock) -> Result<Language, InferError> {
        let on_empty = unit();
        let mut items = VecDeque::from(node.items(self.ast, self.token));
        let Some(last_item) = items.pop_front() else {
            return Ok(on_empty);
        };
        let mut needle = self.convert(last_item)?;

        for item in items {
            let name = item.extract_name(self.ast, self.token);
            let item = self.convert(item)?;

            needle = r#let(var(name), item, needle).into();
        }

        Ok(needle)
    }
}

impl ToModelFrom<InitializedVariable> for ConversionHelper<'_> {
    fn convert(self, node: &InitializedVariable) -> Result<Language, InferError> {
        let expr = self.convert(node.expr(self.ast, self.token))?;
        let scope = self.scopes.lookup(node, self.ast, self.token)?;
        let variable = node.variable(self.ast, self.token);
        let bind = scope
            .var(&variable.name)
            .ok_or(AlgorithmWError::UnknownVar(var(&variable.name)))?;
        Ok(r#let(bind.clone(), expr, bind).into())
    }
}

impl ToModelFrom<Expression> for ConversionHelper<'_> {
    fn convert(self, node: &Expression) -> Result<Language, InferError> {
        match node.as_enum() {
            ExpressionEnum::Lambda(x) => self.convert(x),
            ExpressionEnum::If(x) => self.convert(x),
            ExpressionEnum::Literal(x) => self.convert(x),
            ExpressionEnum::Term(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<IfExpression> for ConversionHelper<'_> {
    fn convert(self, node: &IfExpression) -> Result<Language, InferError> {
        let condition = self.convert(node.condition(self.ast, self.token))?;
        let body = self.convert(node.body(self.ast, self.token))?;
        let otherwise = {
            let elifs = node.elifs(self.ast, self.token);
            let elses = node.elses(self.ast, self.token);
            let last = elses
                .map(|it| self.convert(it.body(self.ast, self.token)))
                .unwrap_or(Ok(unit()))?;

            elifs.into_iter().try_fold(last, |acc, next| {
                let condition = self.convert(next.condition(self.ast, self.token))?;
                let body = self.convert(next.body(self.ast, self.token))?;
                Result::<_, InferError>::Ok(r#if(condition, body, acc).into())
            })?
        };

        Ok(r#if(condition, body, otherwise).into())
    }
}

impl ToModelFrom<Literal> for ConversionHelper<'_> {
    fn convert(self, node: &Literal) -> Result<Language, InferError> {
        match node.as_enum() {
            LiteralEnum::Number(x) => Ok(Floating(x.value.clone()).into()),
            LiteralEnum::Char(x) => Ok(Floating(x.value.clone()).into()),
            LiteralEnum::String(_) => todo!(),
            LiteralEnum::Tuple(node) => {
                let items = node
                    .value(self.ast, self.token)
                    .into_iter()
                    .map(|it| self.convert(it))
                    .collect::<Result<_, _>>()?;
                Ok(Tuple(items).into())
            }
        }
    }
}

impl ToModelFrom<Term> for ConversionHelper<'_> {
    fn convert(self, node: &Term) -> Result<Language, InferError> {
        let TermEnum::Reference(node) = node.as_enum();
        self.convert(node)
    }
}

impl ToModelFrom<Reference> for ConversionHelper<'_> {
    fn convert(self, node: &Reference) -> Result<Language, InferError> {
        let scope = self.scopes.lookup(node, self.ast, self.token)?;
        match &node.ident {
            TypeReference { .. } => Err(InferError::Unknown),
            Identifier::Reference { name } => Ok(scope
                .var(name)
                .ok_or(AlgorithmWError::UnknownVar(var(name)))?
                .into()),
        }
    }
}

impl ToModelFrom<Lambda> for ConversionHelper<'_> {
    fn convert(self, node: &Lambda) -> Result<Language, InferError> {
        let expr = self.convert(node.expr(self.ast, self.token))?;
        let scope = self.scopes.lookup(node, self.ast, self.token)?;
        node.binds(self.ast, self.token)
            .into_iter()
            .map(|it| {
                scope
                    .var(&it.name)
                    .ok_or(AlgorithmWError::UnknownVar(var(&it.name)))
            })
            .try_fold(expr, |acc, next| Ok(lambda(next?, acc).into()))
    }
}

impl ExtractName for BlockLevel {
    fn extract_name(&self, tree: &SyntaxTree, token: &PermTkn) -> Cow<str> {
        match self.as_enum() {
            BlockLevelEnum::Func(x) => x.name.as_str().into(),
            BlockLevelEnum::InitVar(x) => x.variable(tree, token).name.clone().into(),
            BlockLevelEnum::Operation(x) => x.get_id().to_string().into(),
        }
    }
}
