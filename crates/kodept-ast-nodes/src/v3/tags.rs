use kodept_ast::prelude::ASTNode;

pub trait IsDeclaration: ASTNode {}
pub struct Declaration;
pub trait IsStatement<const NORMALIZED: bool = false>: ASTNode {}
pub struct Statement;
pub trait IsExpression: ASTNode {}
pub struct Expression;
pub struct Lhs;
pub struct Rhs;
pub struct Condition;
pub struct Else;

#[derive(Debug)]
pub struct Params;
#[derive(Debug)]
pub struct NamedParams;
#[derive(Debug)]
pub struct ReturnType;
