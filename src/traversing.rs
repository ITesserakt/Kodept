use crate::traversing::OptionalContext::Defined;
use crate::utils;
use kodept_ast::visitor::{gast::GAST, visit_side::VisitSide, GenericASTVisitor};
use kodept_ast::{
    Access, Application, Binary, BlockLevel, BodiedFunctionDeclaration, Body, CharLiteral,
    ElifExpression, ElseExpression, EnumDeclaration, Expression, ExpressionBlock, FileDeclaration,
    Identifier, IfExpression, InitializedVariable, Lambda, Literal, ModuleDeclaration,
    NumberLiteral, Operation, Reference, ResolvedReference, ResolvedTypeReference, StringLiteral,
    StructDeclaration, Term, TopLevel, TupleLiteral, Type, TypeName, TypedParameter, Unary,
    Variable, AST,
};
use kodept_macros::analyzers::ast_node::{ASTNode, ASTNodeMut};
use kodept_macros::erased::Erased;
use kodept_macros::traits::Context;
use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::{DiGraph, NodeIndex};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

type DefaultErased<'c, C, E> = Erased<'c, C, E>;

pub struct TraverseSet<'c, C, E>
where
    C: Context<'c>,
{
    inner: DiGraph<DefaultErased<'c, C, E>, ()>,
}

impl<'c, C: Context<'c>, E> Default for TraverseSet<'c, C, E> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

pub trait Traversable<'c, C: Context<'c>, E> {
    fn traverse(&self, ast: &mut AST, context: C) -> Result<C, (E, C)>;
}

impl<'c, C, E> TraverseSet<'c, C, E>
where
    C: Context<'c>,
{
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn add_independent(&mut self, item: Erased<'c, C, E>) -> NodeIndex {
        self.inner.add_node(item)
    }

    pub fn add_link(&mut self, from: NodeIndex, to: NodeIndex) {
        if from == to {
            panic!("Cannot add link to itself");
        }
        self.inner.add_edge(from, to, ());
        if is_cyclic_directed(&self.inner) {
            panic!("Cannot add link that produces a cycle");
        }
    }

    pub fn add_dependent(&mut self, from: &[NodeIndex], item: Erased<'c, C, E>) -> NodeIndex {
        let id = self.inner.add_node(item);
        from.iter().for_each(|&it| {
            self.inner.add_edge(it, id, ());
        });
        id
    }
}

impl<'c, C: Context<'c>, E> Traversable<'c, C, E> for TraverseSet<'c, C, E> {
    fn traverse(&self, ast: &mut AST, context: C) -> Result<C, (E, C)> {
        let context = RefCell::new(Defined(context));
        let layers = utils::graph::topological_layers(&self.inner);

        for layer in layers {
            let compact_visitor: Vec<_> = layer
                .into_iter()
                .filter_map(|it| self.inner.node_weight(it))
                .map(|it| {
                    let ctx = (it, &context);
                    ErasedNodeVisitor(
                        move |node: ASTNodeMut<'_>, side| {
                            let mut guard = ctx.1.borrow_mut();
                            match ctx.0 {
                                DefaultErased::Transformer(x) => {
                                    x.transform(node, side, &mut guard)
                                }
                                DefaultErased::Analyzer(x) => {
                                    x.analyze(node.into(), side, &mut guard)
                                }
                            }
                        },
                        PhantomData,
                    )
                })
                .collect();

            let gast = GAST::new(compact_visitor);
            ast.apply_visitor(gast)
                .map_err(|e| (e, context.take().into_inner()))?;
        }
        Ok(context.take().into_inner())
    }
}

#[derive(Default)]
enum OptionalContext<C> {
    Defined(C),
    #[default]
    None,
}

impl<C> OptionalContext<C> {
    pub const TAKEN_MSG: &'static str = "Cannot get context that was taken";

    pub fn into_inner(self) -> C {
        match self {
            Defined(c) => c,
            OptionalContext::None => unreachable!("{}", Self::TAKEN_MSG),
        }
    }
}

impl<C> Deref for OptionalContext<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        match self {
            Defined(c) => c,
            OptionalContext::None => unreachable!("{}", OptionalContext::<C>::TAKEN_MSG),
        }
    }
}

impl<C> DerefMut for OptionalContext<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Defined(c) => c,
            OptionalContext::None => unreachable!("{}", OptionalContext::<C>::TAKEN_MSG),
        }
    }
}

struct ErasedNodeVisitor<F, E, N>(F, PhantomData<(E, N)>);

macro_rules! impl_generic_visitor {
    ($generic_node:ident) => {
        impl<'n, F, E> GenericASTVisitor for ErasedNodeVisitor<F, E, $generic_node<'n>>
        where
            F: FnMut($generic_node, VisitSide) -> Result<(), E>,
        {
            type Error = E;

            fn visit_file(
                &mut self,
                node: &mut FileDeclaration,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::File(node), side)
            }

            fn visit_module(
                &mut self,
                node: &mut ModuleDeclaration,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Module(node), side)
            }

            fn visit_top_level(
                &mut self,
                node: &mut TopLevel,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::TopLevel(node), side)
            }

            fn visit_struct(
                &mut self,
                node: &mut StructDeclaration,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Struct(node), side)
            }

            fn visit_enum(
                &mut self,
                node: &mut EnumDeclaration,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Enum(node), side)
            }

            fn visit_typed_parameter(
                &mut self,
                node: &mut TypedParameter,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::TypedParameter(node), side)
            }

            fn visit_type(&mut self, node: &mut Type, side: VisitSide) -> Result<(), Self::Error> {
                self.0($generic_node::Type(node), side)
            }

            fn visit_type_name(
                &mut self,
                node: &mut TypeName,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::TypeName(node), side)
            }

            fn visit_body(&mut self, node: &mut Body, side: VisitSide) -> Result<(), Self::Error> {
                self.0($generic_node::Body(node), side)
            }

            fn visit_bodied_function(
                &mut self,
                node: &mut BodiedFunctionDeclaration,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::BodiedFunction(node), side)
            }

            fn visit_block_level(
                &mut self,
                node: &mut BlockLevel,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::BlockLevel(node), side)
            }

            fn visit_expression_block(
                &mut self,
                node: &mut ExpressionBlock,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::ExpressionBlock(node), side)
            }

            fn visit_init_var(
                &mut self,
                node: &mut InitializedVariable,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::InitializedVariable(node), side)
            }

            fn visit_operation(
                &mut self,
                node: &mut Operation,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Operation(node), side)
            }

            fn visit_variable(
                &mut self,
                node: &mut Variable,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Variable(node), side)
            }

            fn visit_application(
                &mut self,
                node: &mut Application,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Application(node), side)
            }

            fn visit_access(
                &mut self,
                node: &mut Access,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Access(node), side)
            }

            fn visit_unary(
                &mut self,
                node: &mut Unary,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Unary(node), side)
            }

            fn visit_binary(
                &mut self,
                node: &mut Binary,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Binary(node), side)
            }

            fn visit_expr(
                &mut self,
                node: &mut Expression,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Expression(node), side)
            }

            fn visit_lambda(
                &mut self,
                node: &mut Lambda,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Lambda(node), side)
            }

            fn visit_identifier(
                &mut self,
                node: &mut Identifier,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Identifier(node), side)
            }

            fn visit_term(&mut self, node: &mut Term, side: VisitSide) -> Result<(), Self::Error> {
                self.0($generic_node::Term(node), side)
            }

            fn visit_reference(
                &mut self,
                node: &mut Reference,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Reference(node), side)
            }

            fn visit_literal(
                &mut self,
                node: &mut Literal,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Literal(node), side)
            }

            fn visit_number(
                &mut self,
                node: &mut NumberLiteral,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Number(node), side)
            }

            fn visit_char(
                &mut self,
                node: &mut CharLiteral,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Char(node), side)
            }

            fn visit_string(
                &mut self,
                node: &mut StringLiteral,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::String(node), side)
            }

            fn visit_tuple(
                &mut self,
                node: &mut TupleLiteral,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Tuple(node), side)
            }

            fn visit_resolved_type_reference(
                &mut self,
                node: &mut ResolvedTypeReference,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::ResolvedTypeReference(node), side)
            }

            fn visit_resolved_reference(
                &mut self,
                node: &mut ResolvedReference,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::ResolvedReference(node), side)
            }

            fn visit_if(
                &mut self,
                node: &mut IfExpression,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::If(node), side)
            }

            fn visit_elif(
                &mut self,
                node: &mut ElifExpression,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Elif(node), side)
            }

            fn visit_else(
                &mut self,
                node: &mut ElseExpression,
                side: VisitSide,
            ) -> Result<(), Self::Error> {
                self.0($generic_node::Else(node), side)
            }
        }
    };
}

impl_generic_visitor!(ASTNode);
impl_generic_visitor!(ASTNodeMut);
