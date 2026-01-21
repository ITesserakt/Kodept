use kodept_ast::prelude::ASTNode;

pub trait IsDeclaration: ASTNode {}
pub struct Declaration;
pub trait IsStatement: ASTNode {}
pub struct Statement;
pub trait IsExpression: ASTNode {}
pub struct Expression;
pub struct Lhs;
pub struct Rhs;
pub struct Condition;
