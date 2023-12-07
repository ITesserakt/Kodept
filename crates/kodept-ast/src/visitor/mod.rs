use crate::visitor::visit_side::{Skip, VisitSide};
use crate::*;

pub mod gast;
pub mod visit_side;

pub type TraversingResult<E> = Result<(), Skip<E>>;

pub trait GenericASTVisitor {
    type Error;

    fn visit_file(
        &mut self,
        node: &mut FileDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_module(
        &mut self,
        node: &mut ModuleDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_top_level(&mut self, node: &mut TopLevel, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_struct(
        &mut self,
        node: &mut StructDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_enum(
        &mut self,
        node: &mut EnumDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_typed_parameter(
        &mut self,
        node: &mut TypedParameter,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_type(&mut self, node: &mut Type, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_type_name(&mut self, node: &mut TypeName, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_bodied_function(
        &mut self,
        node: &mut BodiedFunctionDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_body(&mut self, node: &mut Body, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_block_level(
        &mut self,
        node: &mut BlockLevel,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_expression_block(
        &mut self,
        node: &mut ExpressionBlock,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_init_var(
        &mut self,
        node: &mut InitializedVariable,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_variable(&mut self, node: &mut Variable, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_operation(&mut self, node: &mut Operation, side: VisitSide)
        -> Result<(), Self::Error>;
    fn visit_application(
        &mut self,
        node: &mut Application,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_access(&mut self, node: &mut Access, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_unary(&mut self, node: &mut Unary, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_binary(&mut self, node: &mut Binary, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_expr(&mut self, node: &mut Expression, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_lambda(&mut self, node: &mut Lambda, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_identifier(
        &mut self,
        node: &mut Identifier,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_term(&mut self, node: &mut Term, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_reference(&mut self, node: &mut Reference, side: VisitSide)
        -> Result<(), Self::Error>;
    fn visit_literal(&mut self, node: &mut Literal, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_number(
        &mut self,
        node: &mut NumberLiteral,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_char(&mut self, node: &mut CharLiteral, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_string(
        &mut self,
        node: &mut StringLiteral,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_tuple(&mut self, node: &mut TupleLiteral, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_resolved_type_reference(
        &mut self,
        node: &mut ResolvedTypeReference,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_resolved_reference(
        &mut self,
        node: &mut ResolvedReference,
        side: VisitSide,
    ) -> Result<(), Self::Error>;
    fn visit_if(&mut self, node: &mut IfExpression, side: VisitSide) -> Result<(), Self::Error>;
    fn visit_elif(&mut self, node: &mut ElifExpression, side: VisitSide)
        -> Result<(), Self::Error>;
    fn visit_else(&mut self, node: &mut ElseExpression, side: VisitSide)
        -> Result<(), Self::Error>;
}

macro_rules! impl_generic_visitor_for_vec {
    [$self:ident => $name:ident($node:ident, $side:ident)] => {{
        for item in $self {
            item.$name($node, $side.clone())?;
        }
        Ok(())
    }};
}

impl<'g, E, G, I> GenericASTVisitor for I
where
    G: GenericASTVisitor<Error = E> + 'g,
    for<'a> &'a mut I: IntoIterator<Item = &'a mut G>,
{
    type Error = E;

    fn visit_file(
        &mut self,
        node: &mut FileDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_file(node, side))
    }

    fn visit_module(
        &mut self,
        node: &mut ModuleDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_module(node, side))
    }

    fn visit_top_level(&mut self, node: &mut TopLevel, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_top_level(node, side))
    }

    fn visit_struct(
        &mut self,
        node: &mut StructDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_struct(node, side))
    }

    fn visit_enum(
        &mut self,
        node: &mut EnumDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_enum(node, side))
    }

    fn visit_typed_parameter(
        &mut self,
        node: &mut TypedParameter,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_typed_parameter(node, side))
    }

    fn visit_type(&mut self, node: &mut Type, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_type(node, side))
    }

    fn visit_type_name(&mut self, node: &mut TypeName, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_type_name(node, side))
    }

    fn visit_bodied_function(
        &mut self,
        node: &mut BodiedFunctionDeclaration,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_bodied_function(node, side))
    }

    fn visit_body(&mut self, node: &mut Body, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_body(node, side))
    }
    fn visit_block_level(
        &mut self,
        node: &mut BlockLevel,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_block_level(node, side))
    }

    fn visit_expression_block(
        &mut self,
        node: &mut ExpressionBlock,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_expression_block(node, side))
    }

    fn visit_init_var(
        &mut self,
        node: &mut InitializedVariable,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_init_var(node, side))
    }

    fn visit_variable(&mut self, node: &mut Variable, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_variable(node, side))
    }

    fn visit_operation(
        &mut self,
        node: &mut Operation,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_operation(node, side))
    }

    fn visit_application(
        &mut self,
        node: &mut Application,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_application(node, side))
    }

    fn visit_access(&mut self, node: &mut Access, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_access(node, side))
    }

    fn visit_unary(&mut self, node: &mut Unary, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_unary(node, side))
    }

    fn visit_binary(&mut self, node: &mut Binary, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_binary(node, side))
    }

    fn visit_expr(&mut self, node: &mut Expression, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_expr(node, side))
    }

    fn visit_lambda(&mut self, node: &mut Lambda, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_lambda(node, side))
    }

    fn visit_identifier(
        &mut self,
        node: &mut Identifier,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_identifier(node, side))
    }

    fn visit_term(&mut self, node: &mut Term, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_term(node, side))
    }

    fn visit_reference(
        &mut self,
        node: &mut Reference,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_reference(node, side))
    }

    fn visit_literal(&mut self, node: &mut Literal, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_literal(node, side))
    }

    fn visit_number(
        &mut self,
        node: &mut NumberLiteral,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_number(node, side))
    }

    fn visit_char(&mut self, node: &mut CharLiteral, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_char(node, side))
    }

    fn visit_string(
        &mut self,
        node: &mut StringLiteral,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_string(node, side))
    }

    fn visit_tuple(&mut self, node: &mut TupleLiteral, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_tuple(node, side))
    }

    fn visit_resolved_type_reference(
        &mut self,
        node: &mut ResolvedTypeReference,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_resolved_type_reference(node, side))
    }

    fn visit_resolved_reference(
        &mut self,
        node: &mut ResolvedReference,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_resolved_reference(node, side))
    }

    fn visit_if(&mut self, node: &mut IfExpression, side: VisitSide) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_if(node, side))
    }

    fn visit_elif(
        &mut self,
        node: &mut ElifExpression,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_elif(node, side))
    }

    fn visit_else(
        &mut self,
        node: &mut ElseExpression,
        side: VisitSide,
    ) -> Result<(), Self::Error> {
        impl_generic_visitor_for_vec!(self => visit_else(node, side))
    }
}
