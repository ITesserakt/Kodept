use crate::node_family::TypeDerivableNode;
use crate::scope::ScopeTree;
use crate::type_checker::{InferError, TypeChecker};
use kodept_ast::graph::{GhostToken, SyntaxTree};
use kodept_ast::traits::Identifiable;
use kodept_ast::{
    BlockLevel, BodiedFunctionDeclaration, Body, Expression, ExpressionBlock, Identifier,
    InitializedVariable, Lambda, Literal, Operation, Term,
};
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::Literal::{Floating, Tuple};
use kodept_inference::language::{app, lambda, r#let, var, Language};
use kodept_inference::r#type::MonomorphicType;
use kodept_inference::Environment;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;
use Identifier::TypeReference;

impl TypeDerivableNode {
    pub fn type_of<'a>(
        &self,
        ast: &'a SyntaxTree,
        token: &'a GhostToken,
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
        if let Some(node) = node.as_function() {
            self.convert(node)
        } else if let Some(node) = node.as_expression_block() {
            self.convert(node)
        } else if let Some(node) = node.as_init_var() {
            self.convert(node)
        } else if let Some(node) = node.as_lambda() {
            self.convert(node)
        } else if let Some(node) = node.as_application() {
            todo!()
        } else if let Some(node) = node.as_if_expr() {
            todo!()
        } else if let Some(node) = node.as_reference() {
            todo!()
        } else if let Some(node) = node.as_literal() {
            self.convert(node)
        } else {
            unreachable!()
        }
    }
}

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

trait ExtractName {
    fn extract_name(&self, tree: &SyntaxTree, token: &GhostToken) -> Cow<str>;
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
                .rev()
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
                TypeReference { .. } => Err(InferError::Unknown),
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

impl ExtractName for BlockLevel {
    fn extract_name(&self, tree: &SyntaxTree, token: &GhostToken) -> Cow<str> {
        if let Some(node) = self.as_func() {
            node.name.as_str().into()
        } else if let Some(node) = self.as_init_var() {
            node.variable(tree, token).name.clone().into()
        } else {
            self.get_id().to_string().into()
        }
    }
}
