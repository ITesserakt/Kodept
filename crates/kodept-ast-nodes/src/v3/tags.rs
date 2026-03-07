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

pub struct Params;
pub struct NamedParams;
pub struct ReturnType;
