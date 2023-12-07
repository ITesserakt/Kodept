use crate::visitor::visit_side::VisitSide;
use crate::visitor::GenericASTVisitor;
use crate::*;
use visita::{Data, NodeFamily, Visit, Visitor};

pub struct GAST<G: GenericASTVisitor>(G);

impl<G: GenericASTVisitor> GAST<G> {
    pub fn new(inner: G) -> Self {
        Self(inner)
    }
}

impl<G: GenericASTVisitor> GAST<G> {
    pub fn into_inner(self) -> G {
        self.0
    }
}

impl<G: GenericASTVisitor> Visitor<FileDeclaration> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<ModuleDeclaration> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<TopLevel> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<StructDeclaration> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<TypedParameter> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<Type> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<EnumDeclaration> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<FunctionDeclaration> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<BodiedFunctionDeclaration> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<Body> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<ExpressionBlock> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<InitializedVariable> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<BlockLevel> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<Operation> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<Expression> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<Lambda> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<Term> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<Literal> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<Identifier> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visitor<IfExpression> for GAST<G> {
    type Output = Result<(), G::Error>;
    type Data<'d> = ();
}

impl<G: GenericASTVisitor> Visit<FileDeclaration, FileDeclaration> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut FileDeclaration,
        data: Data<FileDeclaration, Self>,
    ) -> Self::Output {
        self.0.visit_file(node, VisitSide::Entering)?;
        for module in node.modules.iter_mut() {
            self.visit(module, data)?;
        }
        self.0.visit_file(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<FileDeclaration, ModuleDeclaration> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut ModuleDeclaration,
        data: Data<FileDeclaration, Self>,
    ) -> Self::Output {
        self.0.visit_module(node, VisitSide::Entering)?;
        for item in node.items.iter_mut() {
            self.visit(item, data)?;
        }
        self.0.visit_module(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<ModuleDeclaration, TopLevel> for GAST<G> {
    fn visit(&mut self, node: &mut TopLevel, data: Data<ModuleDeclaration, Self>) -> Self::Output {
        self.0.visit_top_level(node, VisitSide::Entering)?;
        match node {
            TopLevel::Enum(x) => self.visit(x, data),
            TopLevel::Struct(x) => self.visit(x, data),
            TopLevel::Function(x) => self.visit(x, data),
        }?;
        self.0.visit_top_level(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<TopLevel, StructDeclaration> for GAST<G> {
    fn visit(&mut self, node: &mut StructDeclaration, data: Data<TopLevel, Self>) -> Self::Output {
        self.0.visit_struct(node, VisitSide::Entering)?;
        for item in node.parameters.iter_mut() {
            self.visit(item, data)?;
        }
        for item in node.rest.iter_mut() {
            self.visit(item, data)?;
        }
        self.0.visit_struct(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<TopLevel, EnumDeclaration> for GAST<G> {
    fn visit(&mut self, node: &mut EnumDeclaration, data: Data<TopLevel, Self>) -> Self::Output {
        self.0.visit_enum(node, VisitSide::Entering)?;
        for item in node.contents.iter_mut() {
            EnumDeclaration::accept_node(self, item, data)?;
        }
        self.0.visit_enum(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<StructDeclaration, TypedParameter> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut TypedParameter,
        data: Data<StructDeclaration, Self>,
    ) -> Self::Output {
        self.0.visit_typed_parameter(node, VisitSide::Entering)?;
        self.visit(&mut node.parameter_type, data)?;
        self.0.visit_typed_parameter(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<TypedParameter, Type> for GAST<G> {
    fn visit(&mut self, node: &mut Type, data: Data<TypedParameter, Self>) -> Self::Output {
        self.0.visit_type(node, VisitSide::Entering)?;
        match node {
            Type::Reference(x) => Type::accept_node(self, x, data)?,
            Type::Union(SumType { types, .. }) | Type::Tuple(ProdType { types, .. }) => {
                for item in types.iter_mut() {
                    self.visit(item, data)?;
                }
            }
        };
        self.0.visit_type(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Type, TypeName> for GAST<G> {
    fn visit(&mut self, node: &mut TypeName, _data: Data<Type, Self>) -> Self::Output {
        self.0.visit_type_name(node, VisitSide::Leaf)
    }
}

impl<G: GenericASTVisitor> Visit<EnumDeclaration, TypeName> for GAST<G> {
    fn visit(&mut self, node: &mut TypeName, _data: Data<EnumDeclaration, Self>) -> Self::Output {
        self.0.visit_type_name(node, VisitSide::Leaf)
    }
}

impl<G: GenericASTVisitor> Visit<StructDeclaration, BodiedFunctionDeclaration> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut BodiedFunctionDeclaration,
        data: Data<StructDeclaration, Self>,
    ) -> Self::Output {
        self.0.visit_bodied_function(node, VisitSide::Entering)?;
        self.visit(&mut node.body, data)?;
        match &mut node.return_type {
            None => {}
            Some(x) => self.visit(x, data)?,
        };
        self.0.visit_bodied_function(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<BodiedFunctionDeclaration, Body> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut Body,
        data: Data<BodiedFunctionDeclaration, Self>,
    ) -> Self::Output {
        self.0.visit_body(node, VisitSide::Entering)?;
        match node {
            Body::Block(x) => self.visit(x, data),
            Body::Simple(x) => Body::accept_node(self, x.as_mut(), data),
        }?;
        self.0.visit_body(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Body, ExpressionBlock> for GAST<G> {
    fn visit(&mut self, node: &mut ExpressionBlock, data: Data<Body, Self>) -> Self::Output {
        self.0.visit_expression_block(node, VisitSide::Entering)?;
        for item in node.items.iter_mut() {
            self.visit(item, data)?;
        }
        self.0.visit_expression_block(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Body, BlockLevel> for GAST<G> {
    fn visit(&mut self, node: &mut BlockLevel, data: Data<Body, Self>) -> Self::Output {
        self.0.visit_block_level(node, VisitSide::Entering)?;
        match node {
            BlockLevel::InitVar(x) => self.visit(x, data),
            BlockLevel::Operation(x) => self.visit(x, data),
            BlockLevel::Block(x) => self.visit(x, data),
            BlockLevel::Function(x) => self.visit(x, data),
        }?;
        self.0.visit_block_level(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<BlockLevel, InitializedVariable> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut InitializedVariable,
        data: Data<BlockLevel, Self>,
    ) -> Self::Output {
        self.0.visit_init_var(node, VisitSide::Entering)?;
        self.visit(&mut node.variable, data)?;
        self.visit(&mut node.expr, data)?;
        self.0.visit_init_var(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<InitializedVariable, Variable> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut Variable,
        data: Data<InitializedVariable, Self>,
    ) -> Self::Output {
        if node.assigned_type.is_none() {
            self.0.visit_variable(node, VisitSide::Leaf)
        } else {
            self.0.visit_variable(node, VisitSide::Entering)?;
            match node.assigned_type.as_mut() {
                Some(x) => self.visit(x, data),
                None => unreachable!(),
            }?;
            self.0.visit_variable(node, VisitSide::Exiting)
        }
    }
}

impl<G: GenericASTVisitor> Visit<InitializedVariable, Operation> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut Operation,
        data: Data<InitializedVariable, Self>,
    ) -> Self::Output {
        self.0.visit_operation(node, VisitSide::Entering)?;
        match node {
            Operation::Application(x) => self.visit(x.as_mut(), data),
            Operation::Access(x) => self.visit(x.as_mut(), data),
            Operation::Unary(x) => self.visit(x.as_mut(), data),
            Operation::Binary(x) => self.visit(x.as_mut(), data),
            Operation::Expression(x) => self.visit(x.as_mut(), data),
            Operation::Block(x) => self.visit(x.as_mut(), data),
        }?;
        self.0.visit_operation(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Operation, Application> for GAST<G> {
    fn visit(&mut self, node: &mut Application, data: Data<Operation, Self>) -> Self::Output {
        self.0.visit_application(node, VisitSide::Entering)?;
        self.visit(&mut node.expr, data)?;
        for item in &mut node.params {
            self.visit(item, data)?;
        }
        self.0.visit_application(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Operation, Access> for GAST<G> {
    fn visit(&mut self, node: &mut Access, data: Data<Operation, Self>) -> Self::Output {
        self.0.visit_access(node, VisitSide::Entering)?;
        self.visit(&mut node.left, data)?;
        self.visit(&mut node.right, data)?;
        self.0.visit_access(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Operation, Unary> for GAST<G> {
    fn visit(&mut self, node: &mut Unary, data: Data<Operation, Self>) -> Self::Output {
        self.0.visit_unary(node, VisitSide::Entering)?;
        self.visit(&mut node.expr, data)?;
        self.0.visit_unary(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Operation, Binary> for GAST<G> {
    fn visit(&mut self, node: &mut Binary, data: Data<Operation, Self>) -> Self::Output {
        self.0.visit_binary(node, VisitSide::Entering)?;
        self.visit(&mut node.left, data)?;
        self.visit(&mut node.right, data)?;
        self.0.visit_binary(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Operation, Expression> for GAST<G> {
    fn visit(&mut self, node: &mut Expression, data: Data<Operation, Self>) -> Self::Output {
        self.0.visit_expr(node, VisitSide::Entering)?;
        match node {
            Expression::Lambda(x) => self.visit(x, data),
            Expression::Term(x) => self.visit(x, data),
            Expression::Literal(x) => self.visit(x, data),
            Expression::If(x) => self.visit(x, data),
        }?;
        self.0.visit_expr(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Expression, Lambda> for GAST<G> {
    fn visit(&mut self, node: &mut Lambda, data: Data<Expression, Self>) -> Self::Output {
        self.0.visit_lambda(node, VisitSide::Entering)?;
        self.visit(&mut node.expr, data)?;
        for item in &mut node.binds {
            self.visit(item, data)?;
        }
        self.0.visit_lambda(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Lambda, Identifier> for GAST<G> {
    fn visit(&mut self, node: &mut Identifier, data: Data<Lambda, Self>) -> Self::Output {
        match node {
            Identifier::TypeReference { .. } => self.0.visit_identifier(node, VisitSide::Leaf),
            Identifier::Reference { .. } => self.0.visit_identifier(node, VisitSide::Leaf),
            Identifier::ResolvedTypeReference(x) => self.visit(x, data),
            Identifier::ResolvedReference(x) => self.visit(x, data),
        }
    }
}

impl<G: GenericASTVisitor> Visit<Expression, Term> for GAST<G> {
    fn visit(&mut self, node: &mut Term, data: Data<Expression, Self>) -> Self::Output {
        self.0.visit_term(node, VisitSide::Entering)?;
        match node {
            Term::Reference(x) => self.visit(x, data),
        }?;
        self.0.visit_term(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Term, Reference> for GAST<G> {
    fn visit(&mut self, node: &mut Reference, data: Data<Term, Self>) -> Self::Output {
        self.0.visit_reference(node, VisitSide::Entering)?;
        self.visit(&mut node.ident, data)?;
        self.0.visit_reference(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Expression, Literal> for GAST<G> {
    fn visit(&mut self, node: &mut Literal, data: Data<Expression, Self>) -> Self::Output {
        self.0.visit_literal(node, VisitSide::Entering)?;
        match node {
            Literal::Number(x) => self.visit(x, data),
            Literal::Char(x) => self.visit(x, data),
            Literal::String(x) => self.visit(x, data),
            Literal::Tuple(x) => self.visit(x, data),
        }?;
        self.0.visit_literal(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Literal, NumberLiteral> for GAST<G> {
    fn visit(&mut self, node: &mut NumberLiteral, _data: Data<Literal, Self>) -> Self::Output {
        self.0.visit_number(node, VisitSide::Leaf)
    }
}

impl<G: GenericASTVisitor> Visit<Literal, CharLiteral> for GAST<G> {
    fn visit(&mut self, node: &mut CharLiteral, _data: Data<Literal, Self>) -> Self::Output {
        self.0.visit_char(node, VisitSide::Leaf)
    }
}

impl<G: GenericASTVisitor> Visit<Literal, StringLiteral> for GAST<G> {
    fn visit(&mut self, node: &mut StringLiteral, _data: Data<Literal, Self>) -> Self::Output {
        self.0.visit_string(node, VisitSide::Leaf)
    }
}

impl<G: GenericASTVisitor> Visit<Literal, TupleLiteral> for GAST<G> {
    fn visit(&mut self, node: &mut TupleLiteral, data: Data<Literal, Self>) -> Self::Output {
        self.0.visit_tuple(node, VisitSide::Entering)?;
        for item in &mut node.value {
            self.visit(item, data)?;
        }
        self.0.visit_tuple(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Identifier, ResolvedTypeReference> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut ResolvedTypeReference,
        data: Data<Identifier, Self>,
    ) -> Self::Output {
        self.0
            .visit_resolved_type_reference(node, VisitSide::Entering)?;
        self.visit(&mut node.reference_type, data)?;
        self.0
            .visit_resolved_type_reference(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<Identifier, ResolvedReference> for GAST<G> {
    fn visit(
        &mut self,
        node: &mut ResolvedReference,
        _data: Data<Identifier, Self>,
    ) -> Self::Output {
        self.0.visit_resolved_reference(node, VisitSide::Leaf)
    }
}

impl<G: GenericASTVisitor> Visit<Expression, IfExpression> for GAST<G> {
    fn visit(&mut self, node: &mut IfExpression, data: Data<Expression, Self>) -> Self::Output {
        self.0.visit_if(node, VisitSide::Entering)?;
        self.visit(&mut node.condition, data)?;
        self.visit(&mut node.body, data)?;
        for item in &mut node.elif {
            self.visit(item, data)?;
        }
        match node.el.as_mut() {
            None => {}
            Some(x) => self.visit(x, data)?,
        };
        self.0.visit_if(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<IfExpression, ElifExpression> for GAST<G> {
    fn visit(&mut self, node: &mut ElifExpression, data: Data<IfExpression, Self>) -> Self::Output {
        self.0.visit_elif(node, VisitSide::Entering)?;
        self.visit(&mut node.condition, data)?;
        self.visit(&mut node.body, data)?;
        self.0.visit_elif(node, VisitSide::Exiting)
    }
}

impl<G: GenericASTVisitor> Visit<IfExpression, ElseExpression> for GAST<G> {
    fn visit(&mut self, node: &mut ElseExpression, data: Data<IfExpression, Self>) -> Self::Output {
        self.0.visit_else(node, VisitSide::Entering)?;
        self.visit(&mut node.body, data)?;
        self.0.visit_else(node, VisitSide::Exiting)
    }
}
