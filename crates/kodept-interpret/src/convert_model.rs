use std::borrow::Cow;

use nonempty_collections::nev;

use Identifier::TypeReference;
use kodept_ast::{
    Appl, BlockLevel, BlockLevelEnum, Body, BodyEnum, BodyFnDecl, CodeFlowEnum, Expression,
    ExpressionEnum, Exprs, Identifier, IfExpr, InitVar, Lambda, Lit, LitEnum, Operation,
    OperationEnum, ParamEnum, Ref, Term, TermEnum,
};
use kodept_ast::graph::{PermTkn, SyntaxTree};
use kodept_ast::traits::{AsEnum, Identifiable};
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::language::{app, bounded, BVar, lambda, Language, r#if, r#let, var};
use kodept_inference::language::Literal::{Floating, Tuple};

use crate::node_family::convert;
use crate::scope::ScopeTree;
use crate::type_checker::{InferError, TypeChecker};
use crate::type_checker::InferError::Unknown;
use crate::Witness;

impl TypeChecker<'_> {
    #[allow(private_bounds)]
    pub(crate) fn to_model<'a, N>(
        &'a self,
        ast: &'a SyntaxTree,
        token: &'a PermTkn,
        node: &N,
        evidence: Witness,
    ) -> Result<Language, InferError>
    where
        ConversionHelper<'a>: ToModelFrom<N>,
    {
        let helper = ConversionHelper {
            scopes: &self.symbols,
            ast,
            token,
            evidence,
        };
        helper.convert(node)
    }
}

#[derive(Copy, Clone)]
struct ConversionHelper<'a> {
    scopes: &'a ScopeTree,
    ast: &'a SyntaxTree,
    token: &'a PermTkn,
    evidence: Witness,
}

#[inline]
const fn unit() -> Language {
    Language::Literal(Tuple(vec![]))
}

trait ToModelFrom<N> {
    fn convert(self, node: &N) -> Result<Language, InferError>;
}

pub(crate) trait ExtractName {
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
            BlockLevelEnum::Fn(x) => self.convert(x),
            BlockLevelEnum::InitVar(x) => self.convert(x),
            BlockLevelEnum::Op(x) => self.convert(x),
            BlockLevelEnum::Block(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<BodyFnDecl> for ConversionHelper<'_> {
    fn convert(self, node: &BodyFnDecl) -> Result<Language, InferError> {
        let expr = self.convert(node.body(self.ast, self.token))?;
        let scope = self.scopes.lookup(node, self.ast, self.token)?;
        let mut bindings = node
            .parameters(self.ast, self.token)
            .into_iter()
            .map(|it| {
                let (name, ty) = match it.as_enum() {
                    ParamEnum::Ty(x) => (&x.name, Some(x.parameter_type(self.ast, self.token))),
                    ParamEnum::NonTy(x) => (&x.name, None),
                };
                let assigned_ty = match ty {
                    None => None,
                    Some(ty) => Some(convert(ty, scope.clone(), self.ast, self.token)?),
                };
                let var = scope
                    .var(name)
                    .ok_or(AlgorithmWError::UnknownVar(nev![var(name)]))?;
                Ok(match assigned_ty {
                    None => var.into(),
                    Some(ty) => bounded(var, ty),
                })
            })
            .peekable();
        if bindings.peek().is_some() {
            bindings.try_rfold(expr, |acc, next: Result<BVar, InferError>| {
                Ok(lambda(next?, acc).into())
            })
        } else {
            Ok(expr)
        }
    }
}

impl ToModelFrom<Operation> for ConversionHelper<'_> {
    fn convert(self, node: &Operation) -> Result<Language, InferError> {
        match node.as_enum() {
            OperationEnum::Appl(node) => self.convert(node),
            OperationEnum::Acc(_) => self.evidence.prove(),
            OperationEnum::Unary(_) => self.evidence.prove(),
            OperationEnum::Binary(_) => self.evidence.prove(),
            OperationEnum::Block(x) => self.convert(x),
            OperationEnum::Expr(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<Appl> for ConversionHelper<'_> {
    fn convert(self, node: &Appl) -> Result<Language, InferError> {
        let expr = self.convert(node.expr(self.ast, self.token))?;
        let mut params = node
            .params(self.ast, self.token)
            .into_iter()
            .rev()
            .map(|it| self.convert(it))
            .peekable();
        if params.peek().is_some() {
            params.try_rfold(expr, |acc, next| Ok(app(next?, acc).into()))
        } else {
            Ok(app(unit(), expr).into())
        }
    }
}

impl ToModelFrom<Exprs> for ConversionHelper<'_> {
    fn convert(self, node: &Exprs) -> Result<Language, InferError> {
        let on_empty = unit();
        let mut items = node.items(self.ast, self.token);
        let Some(last_item) = items.pop() else {
            return Ok(on_empty);
        };

        let needle = self.convert(last_item)?;
        items.into_iter().try_rfold(needle, |needle, item| {
            if let BlockLevelEnum::InitVar(v) = item.as_enum() {
                if let Language::Let(l) = self.convert(v)? {
                    return Ok(r#let(l.bind, *l.binder, needle).into());
                }
            }

            let name = item.extract_name(self.ast, self.token);
            let item = self.convert(item)?;

            Ok(r#let(var(name), item, needle).into())
        })
    }
}

impl ToModelFrom<InitVar> for ConversionHelper<'_> {
    fn convert(self, node: &InitVar) -> Result<Language, InferError> {
        let expr = self.convert(node.expr(self.ast, self.token))?;
        let scope = self.scopes.lookup(node, self.ast, self.token)?;
        let variable = node.variable(self.ast, self.token);
        let bind = scope
            .var(&variable.name)
            .ok_or(AlgorithmWError::UnknownVar(nev![var(&variable.name)]))?;
        let assigned_ty = variable
            .assigned_type(self.ast, self.token)
            .map(|it| convert(it, scope, self.ast, self.token))
            .map_or(Ok(None), |it| it.map(Some))?;
        let var = assigned_ty
            .map(|it| bounded(bind.clone(), it))
            .unwrap_or(bind.clone().into());
        Ok(r#let(var, expr, bind).into())
    }
}

impl ToModelFrom<Expression> for ConversionHelper<'_> {
    fn convert(self, node: &Expression) -> Result<Language, InferError> {
        match node.as_enum() {
            ExpressionEnum::Lambda(x) => self.convert(x),
            ExpressionEnum::CodeFlow(x) => match x.as_enum() {
                CodeFlowEnum::If(x) => self.convert(x),
            },
            ExpressionEnum::Lit(x) => self.convert(x),
            ExpressionEnum::Term(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<IfExpr> for ConversionHelper<'_> {
    fn convert(self, node: &IfExpr) -> Result<Language, InferError> {
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

impl ToModelFrom<Lit> for ConversionHelper<'_> {
    fn convert(self, node: &Lit) -> Result<Language, InferError> {
        match node.as_enum() {
            LitEnum::Num(x) => Ok(Floating(x.value.clone()).into()),
            LitEnum::Char(x) => Ok(Floating(x.value.clone()).into()),
            LitEnum::Str(_) => todo!(),
            LitEnum::Tuple(node) => {
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
        let TermEnum::Ref(node) = node.as_enum();
        self.convert(node)
    }
}

impl ToModelFrom<Ref> for ConversionHelper<'_> {
    fn convert(self, node: &Ref) -> Result<Language, InferError> {
        let scope = self.scopes.lookup(node, self.ast, self.token)?;
        match &node.ident {
            TypeReference { .. } => Err(Unknown),
            Identifier::Reference { name } => Ok(scope
                .var(name)
                .ok_or(AlgorithmWError::UnknownVar(nev![var(name)]))?
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
                    .ok_or(AlgorithmWError::UnknownVar(nev![var(&it.name)]))
            })
            .try_fold(expr, |acc, next| Ok(lambda(next?, acc).into()))
    }
}

impl ExtractName for BlockLevel {
    fn extract_name(&self, tree: &SyntaxTree, token: &PermTkn) -> Cow<str> {
        match self.as_enum() {
            BlockLevelEnum::Fn(x) => x.extract_name(tree, token),
            BlockLevelEnum::InitVar(x) => x.variable(tree, token).name.clone().into(),
            _ => self.get_id().to_string().into(),
        }
    }
}

impl ExtractName for BodyFnDecl {
    fn extract_name(&self, _: &SyntaxTree, _: &PermTkn) -> Cow<str> {
        self.name.as_str().into()
    }
}
