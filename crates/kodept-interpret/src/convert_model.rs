use crate::scope::ScopeTree;
use crate::type_checker::{InferError, TypeChecker};
use kodept_ast::graph::{GhostToken, SyntaxTree};
use kodept_ast::{
    BlockLevel, BodiedFunctionDeclaration, Body, Expression, ExpressionBlock, Identifier,
    InitializedVariable, Lambda, Literal, Operation, Term,
};
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::language::Literal::{Floating, Tuple};
use kodept_inference::language::{app, lambda, r#let, var, Language};
use tracing::debug;

impl TypeChecker {
    #[allow(private_bounds)]
    pub(crate) fn to_model<'a, N>(
        &'a self,
        ast: &'a SyntaxTree,
        token: &'a GhostToken,
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
    token: &'a GhostToken,
}

#[inline]
const fn unit() -> Language {
    Language::Literal(Tuple(vec![]))
}

trait ToModelFrom<N> {
    fn convert(self, node: &N) -> Result<Language, InferError>;
}

impl ToModelFrom<Body> for ConversionHelper<'_> {
    fn convert(self, node: &Body) -> Result<Language, InferError> {
        if let Some(node) = node.as_simple() {
            return self.convert(node);
        } else if let Some(node) = node.as_block() {
            return self.convert(node);
        }
        unreachable!()
    }
}

impl ToModelFrom<BlockLevel> for ConversionHelper<'_> {
    fn convert(self, node: &BlockLevel) -> Result<Language, InferError> {
        if let Some(node) = node.as_func() {
            return self.convert(node);
        } else if let Some(node) = node.as_operation() {
            return self.convert(node);
        } else if let Some(node) = node.as_init_var() {
            return self.convert(node);
        }
        unreachable!()
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
                let name = if let Some(it) = it.as_typed() {
                    &it.name
                } else if let Some(it) = it.as_untyped() {
                    &it.name
                } else {
                    unreachable!()
                };
                scope
                    .var(name)
                    .ok_or(AlgorithmWError::UnknownVar(var(name)))
            })
            .peekable();
        if bindings.peek().is_some() {
            bindings.try_fold(expr, |acc, next| Ok(lambda(next?, acc).into()))
        } else {
            Ok(lambda(var("()"), expr).into())
        }
    }
}

impl ToModelFrom<Operation> for ConversionHelper<'_> {
    fn convert(self, node: &Operation) -> Result<Language, InferError> {
        if let Some(node) = node.as_block() {
            return self.convert(node);
        } else if let Some(node) = node.as_expression() {
            return self.convert(node);
        } else if let Some(node) = node.as_application() {
            let expr = self.convert(node.expr(self.ast, self.token))?;
            let mut params = node
                .params(self.ast, self.token)
                .into_iter()
                .map(|it| self.convert(it))
                .peekable();
            return if params.peek().is_some() {
                params.try_fold(expr, |acc, next| Ok(app(next?, acc).into()))
            } else {
                Ok(app(unit(), expr).into())
            };
        } else if let Some(node) = node.as_access() {
        } else if let Some(node) = node.as_binary() {
        } else if let Some(node) = node.as_unary() {
        }
        unreachable!()
    }
}

impl ToModelFrom<ExpressionBlock> for ConversionHelper<'_> {
    fn convert(self, node: &ExpressionBlock) -> Result<Language, InferError> {
        node.items(self.ast, self.token)
            .into_iter()
            .map(|it| self.convert(it))
            .enumerate()
            .try_fold(unit(), |acc, (idx, next)| {
                let next = next?;
                debug!("{acc}");
                if let Language::Let(l) = next {
                    Ok(r#let(l.bind.clone(), l, acc).into())
                } else {
                    Ok(r#let(var(idx.to_string()), next, acc).into())
                }
            })
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
        Ok(r#let(bind, expr, unit()).into())
    }
}

impl ToModelFrom<Expression> for ConversionHelper<'_> {
    fn convert(self, node: &Expression) -> Result<Language, InferError> {
        if let Some(node) = node.as_term() {
            return self.convert(node);
        } else if let Some(node) = node.as_if() {
        } else if let Some(node) = node.as_literal() {
            return self.convert(node);
        } else if let Some(node) = node.as_lambda() {
            return self.convert(node);
        }
        unreachable!()
    }
}

impl ToModelFrom<Literal> for ConversionHelper<'_> {
    fn convert(self, node: &Literal) -> Result<Language, InferError> {
        if let Some(node) = node.as_number() {
            return Ok(Floating(node.value.clone()).into());
        } else if let Some(node) = node.as_tuple() {
            let items = node
                .value(self.ast, self.token)
                .into_iter()
                .map(|it| self.convert(it))
                .collect::<Result<_, _>>()?;
            return Ok(Tuple(items).into());
        }
        unreachable!()
    }
}

impl ToModelFrom<Term> for ConversionHelper<'_> {
    fn convert(self, node: &Term) -> Result<Language, InferError> {
        if let Some(node) = node.as_reference() {
            let scope = self.scopes.lookup(node, self.ast, self.token)?;
            return match &node.ident {
                Identifier::TypeReference { .. } => Err(InferError::Unknown),
                Identifier::Reference { name } => Ok(scope
                    .var(name)
                    .ok_or(AlgorithmWError::UnknownVar(var(name)))?
                    .into()),
            };
        }
        unreachable!()
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
