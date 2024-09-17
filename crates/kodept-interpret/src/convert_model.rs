use std::borrow::Cow;

use nonempty_collections::nev;

use crate::node_family::convert;
use crate::scope::ScopeTree;
use crate::type_checker::InferError;
use kodept_ast::graph::stage::FullAccess;
use kodept_ast::graph::SyntaxTree;
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_ast::traits::{AsEnum, Identifiable};
use kodept_ast::{
    node_sub_enum, Appl, BlockLevel, BlockLevelEnum, Body, BodyEnum, BodyFnDecl, CodeFlowEnum,
    Expression, ExpressionEnum, Exprs, Identifier, IfExpr, InitVar, Lambda, Lit, LitEnum,
    Operation, OperationEnum, ParamEnum, Ref, Term, TermEnum,
};
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::language::Literal::{Floating, Tuple};
use kodept_inference::language::{app, bounded, lambda, r#if, r#let, var, BVar, Language};
use kodept_macros::error::traits::SpannedError;
use Identifier::TypeReference;

node_sub_enum! {
    pub(crate) enum ModelConvertibleNode {
        Body(forward Body),
        Block(forward BlockLevel),
        Fn(BodyFnDecl),
        Op(forward Operation),
        Appl(Appl),
        Exprs(Exprs),
        InitVar(InitVar),
        Expr(forward Expression),
        If(IfExpr),
        Lit(forward Lit),
        Term(forward Term),
        Ref(Ref),
        Lambda(Lambda)
    }
}

impl ModelConvertibleNode {
    pub(crate) fn to_model(
        &self,
        scopes: &ScopeTree,
        ast: &SyntaxTree<FullAccess>,
        rlt: &RLTAccessor,
    ) -> Result<Language, SpannedError<InferError>> {
        let helper = ConversionHelper { scopes, ast, rlt };

        match self.as_enum() {
            ModelConvertibleNodeEnum::Body(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Block(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Fn(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Op(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Appl(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Exprs(x) => helper.convert(x),
            ModelConvertibleNodeEnum::InitVar(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Expr(x) => helper.convert(x),
            ModelConvertibleNodeEnum::If(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Lit(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Term(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Ref(x) => helper.convert(x),
            ModelConvertibleNodeEnum::Lambda(x) => helper.convert(x),
        }
    }
}

#[derive(Copy, Clone)]
struct ConversionHelper<'a> {
    scopes: &'a ScopeTree,
    ast: &'a SyntaxTree<FullAccess>,
    rlt: &'a RLTAccessor<'a>,
}

#[inline]
const fn unit() -> Language {
    Language::Literal(Tuple(vec![]))
}

trait ToModelFrom<N> {
    fn convert(self, node: &N) -> Result<Language, SpannedError<InferError>>;
}

pub(crate) trait ExtractName {
    fn extract_name(&self, tree: &SyntaxTree<FullAccess>) -> Cow<str>;
}

impl ToModelFrom<Body> for ConversionHelper<'_> {
    fn convert(self, node: &Body) -> Result<Language, SpannedError<InferError>> {
        match node.as_enum() {
            BodyEnum::Block(x) => self.convert(x),
            BodyEnum::Simple(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<BlockLevel> for ConversionHelper<'_> {
    fn convert(self, node: &BlockLevel) -> Result<Language, SpannedError<InferError>> {
        match node.as_enum() {
            BlockLevelEnum::Fn(x) => self.convert(x),
            BlockLevelEnum::InitVar(x) => self.convert(x),
            BlockLevelEnum::Op(x) => self.convert(x),
            BlockLevelEnum::Block(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<BodyFnDecl> for ConversionHelper<'_> {
    fn convert(self, node: &BodyFnDecl) -> Result<Language, SpannedError<InferError>> {
        let expr = self.convert(node.body(self.ast))?;
        let scope = self
            .scopes
            .lookup(node, self.ast)
            .map_err(|e| SpannedError::for_node(InferError::Scope(e), node.get_id(), self.rlt))?;
        let mut bindings = node
            .parameters(self.ast)
            .into_iter()
            .map(|it| {
                let (name, ty) = match it.as_enum() {
                    ParamEnum::Ty(x) => (&x.name, Some(x.parameter_type(self.ast))),
                    ParamEnum::NonTy(x) => (&x.name, None),
                };
                let assigned_ty = match ty {
                    None => None,
                    Some(ty) => Some(convert(ty, &scope, self.ast, self.rlt)?),
                };
                let var = scope
                    .var(name)
                    .ok_or(AlgorithmWError::UnknownVar(nev![var(name)]))
                    .map_err(|e| {
                        SpannedError::for_node(InferError::AlgorithmW(e), it.get_id(), self.rlt)
                    })?;
                Ok(match assigned_ty {
                    None => var.into(),
                    Some(ty) => bounded(var, ty),
                })
            })
            .peekable();
        if bindings.peek().is_some() {
            bindings.try_rfold(expr, |acc, next: Result<BVar, SpannedError<_>>| {
                Ok(lambda(next?, acc).into())
            })
        } else {
            Ok(expr)
        }
    }
}

impl ToModelFrom<Operation> for ConversionHelper<'_> {
    fn convert(self, node: &Operation) -> Result<Language, SpannedError<InferError>> {
        match node.as_enum() {
            OperationEnum::Appl(node) => self.convert(node),
            OperationEnum::Acc(_) => unreachable!(),
            OperationEnum::Unary(_) => unreachable!(),
            OperationEnum::Binary(_) => unreachable!(),
            OperationEnum::Block(x) => self.convert(x),
            OperationEnum::Expr(x) => self.convert(x),
        }
    }
}

impl ToModelFrom<Appl> for ConversionHelper<'_> {
    fn convert(self, node: &Appl) -> Result<Language, SpannedError<InferError>> {
        let expr = self.convert(node.expr(self.ast))?;
        let mut params = node
            .params(self.ast)
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
    fn convert(self, node: &Exprs) -> Result<Language, SpannedError<InferError>> {
        let on_empty = unit();
        let mut items = node.items(self.ast);
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

            let name = item.extract_name(self.ast);
            let item = self.convert(item)?;

            Ok(r#let(var(name), item, needle).into())
        })
    }
}

impl ToModelFrom<InitVar> for ConversionHelper<'_> {
    fn convert(self, node: &InitVar) -> Result<Language, SpannedError<InferError>> {
        let expr = self.convert(node.expr(self.ast))?;
        let scope = self
            .scopes
            .lookup(node, self.ast)
            .map_err(|e| SpannedError::for_node(InferError::Scope(e), node.get_id(), self.rlt))?;
        let variable = node.variable(self.ast);
        let bind = scope
            .var(&variable.name)
            .ok_or(AlgorithmWError::UnknownVar(nev![var(&variable.name)]))
            .map_err(|e| {
                SpannedError::for_node(InferError::AlgorithmW(e), variable.get_id(), self.rlt)
            })?;
        let assigned_ty = variable
            .assigned_type(self.ast)
            .map(|it| convert(it, &scope, self.ast, self.rlt))
            .map_or(Ok(None), |it| it.map(Some))?;
        let var = assigned_ty
            .map(|it| bounded(bind.clone(), it))
            .unwrap_or(bind.clone().into());
        Ok(r#let(var, expr, bind).into())
    }
}

impl ToModelFrom<Expression> for ConversionHelper<'_> {
    fn convert(self, node: &Expression) -> Result<Language, SpannedError<InferError>> {
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
    fn convert(self, node: &IfExpr) -> Result<Language, SpannedError<InferError>> {
        let condition = self.convert(node.condition(self.ast))?;
        let body = self.convert(node.body(self.ast))?;
        let otherwise = {
            let elifs = node.elifs(self.ast);
            let elses = node.elses(self.ast);
            let last = elses
                .map(|it| self.convert(it.body(self.ast)))
                .unwrap_or(Ok(unit()))?;

            elifs.into_iter().try_fold(last, |acc, next| {
                let condition = self.convert(next.condition(self.ast))?;
                let body = self.convert(next.body(self.ast))?;
                Result::<_, _>::Ok(r#if(condition, body, acc).into())
            })?
        };

        Ok(r#if(condition, body, otherwise).into())
    }
}

impl ToModelFrom<Lit> for ConversionHelper<'_> {
    fn convert(self, node: &Lit) -> Result<Language, SpannedError<InferError>> {
        match node.as_enum() {
            LitEnum::Num(x) => Ok(Floating(x.value.clone()).into()),
            LitEnum::Char(x) => Ok(Floating(x.value.clone()).into()),
            LitEnum::Str(_) => todo!(),
            LitEnum::Tuple(node) => {
                let items = node
                    .value(self.ast)
                    .into_iter()
                    .map(|it| self.convert(it))
                    .collect::<Result<_, _>>()?;
                Ok(Tuple(items).into())
            }
        }
    }
}

impl ToModelFrom<Term> for ConversionHelper<'_> {
    fn convert(self, node: &Term) -> Result<Language, SpannedError<InferError>> {
        let TermEnum::Ref(node) = node.as_enum();
        self.convert(node)
    }
}

impl ToModelFrom<Ref> for ConversionHelper<'_> {
    fn convert(self, node: &Ref) -> Result<Language, SpannedError<InferError>> {
        let scope = self
            .scopes
            .lookup(node, self.ast)
            .map_err(|e| SpannedError::for_node(InferError::Scope(e), node.get_id(), self.rlt))?;
        match &node.ident {
            TypeReference { .. } => todo!("Too complex"),
            Identifier::Reference { name } => scope
                .var(name)
                .map(|it| it.into())
                .ok_or(AlgorithmWError::UnknownVar(nev![var(name)]))
                .map_err(|e| {
                    SpannedError::for_node(InferError::AlgorithmW(e), node.get_id(), self.rlt)
                }),
        }
    }
}

impl ToModelFrom<Lambda> for ConversionHelper<'_> {
    fn convert(self, node: &Lambda) -> Result<Language, SpannedError<InferError>> {
        let expr = self.convert(node.expr(self.ast))?;
        let scope = self
            .scopes
            .lookup(node, self.ast)
            .map_err(|e| SpannedError::for_node(InferError::Scope(e), node.get_id(), self.rlt))?;
        node.binds(self.ast)
            .into_iter()
            .map(|it| {
                scope
                    .var(it.name())
                    .ok_or(AlgorithmWError::UnknownVar(nev![var(it.name())]))
                    .map_err(|e| {
                        SpannedError::for_node(InferError::AlgorithmW(e), it.get_id(), self.rlt)
                    })
            })
            .try_fold(expr, |acc, next| Ok(lambda(next?, acc).into()))
    }
}

impl ExtractName for BlockLevel {
    fn extract_name(&self, tree: &SyntaxTree<FullAccess>) -> Cow<str> {
        match self.as_enum() {
            BlockLevelEnum::Fn(x) => x.extract_name(tree),
            BlockLevelEnum::InitVar(x) => x.variable(tree).name.clone().into(),
            _ => self.get_id().to_string().into(),
        }
    }
}

impl ExtractName for BodyFnDecl {
    fn extract_name(&self, _: &SyntaxTree<FullAccess>) -> Cow<str> {
        self.name.as_str().into()
    }
}
