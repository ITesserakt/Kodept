use crate::Modules;
use crate::v3::Link;
use crate::v3::tags::{
    Condition, Declaration, Expression, IsDeclaration, IsExpression, IsStatement, Lhs, Rhs,
    Statement,
};
use crate::v3::types::{
    AnonFunction, Block, Branch, Call, ForeignFunction, If, Literal, Module, NameRef, Otherwise,
    PrimType, Tuple, TypeRef, UserFunction, UserType, Value, ValueCtor, Variable,
};
use kodept_ast::arity::{Optional, Plural, Singular};
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{HasProperty, Name, RequireProperty};
use kodept_ast::syntax_tree::children::{Family, HasChild};

impl ASTNode for Modules {}
impl Family for Modules {
    type Arity = Plural;
}
impl HasChild<Module, ()> for Modules {}

impl ASTNode for Module {}
impl RequireProperty<Name> for Module {}
impl Family<Declaration> for Module {
    type Arity = Plural;
}
impl<T: IsDeclaration> HasChild<T, Declaration> for Module {}

impl ASTNode for UserType {}
impl IsDeclaration for UserType {}
impl HasProperty<Name> for UserType {}
impl Family for UserType {
    type Arity = Plural;
}
impl Family<Declaration> for UserType {
    type Arity = Plural;
}
impl<T: TypeRef<true>> HasChild<ValueCtor<T>, ()> for UserType {}
impl<T: TypeRef<false>> HasChild<UserFunction<T>, Declaration> for UserType {}

impl<T: TypeRef<true>> ASTNode for ValueCtor<T> {}

impl ASTNode for PrimType {}
impl IsDeclaration for PrimType {}

impl<T: TypeRef<false>> ASTNode for UserFunction<T> {}
impl<T: TypeRef<false>> IsDeclaration for UserFunction<T> {}
impl<T: TypeRef<false>> IsStatement for UserFunction<T> {}
impl<T: TypeRef<false>> IsStatement<true> for UserFunction<T> {}
impl<T: TypeRef<false>> RequireProperty<Name> for UserFunction<T> {}
impl<U: TypeRef<false>> Family for UserFunction<U> {
    type Arity = Singular;
}
impl<U: TypeRef<false>> HasChild<Block, ()> for UserFunction<U> {}

impl<T: TypeRef<true>> ASTNode for ForeignFunction<T> {}
impl<T: TypeRef<true>> IsDeclaration for ForeignFunction<T> {}
impl<T: TypeRef<true>> RequireProperty<Name> for ForeignFunction<T> {}

impl<T: TypeRef<false>> ASTNode for AnonFunction<T> {}
impl<T: TypeRef<false>> IsExpression for AnonFunction<T> {}
impl<T: TypeRef<false>> IsStatement for AnonFunction<T> {}
impl<T: TypeRef<false>> Family for AnonFunction<T> {
    type Arity = Singular;
}
impl<T: TypeRef<false>> HasChild<Block, ()> for AnonFunction<T> {}

impl<T: TypeRef<false>> ASTNode for Variable<T> {}
impl<T: TypeRef<false>> IsStatement for Variable<T> {}
impl<T: TypeRef<false>> IsStatement<true> for Variable<T> {}
impl<T: TypeRef<false>> RequireProperty<Name> for Variable<T> {}
impl<T: TypeRef<false>> Family<Expression> for Variable<T> {
    type Arity = Singular;
}
impl<T: IsExpression, U: TypeRef<false>> HasChild<T, Expression> for Variable<U> {}

impl<const NORMALIZED: bool> ASTNode for Block<NORMALIZED> {}
impl<const NORMALIZED: bool> IsStatement<NORMALIZED> for Block<NORMALIZED> {}
impl<const NORMALIZED: bool> IsExpression for Block<NORMALIZED> {}
impl<const NORMALIZED: bool> Family<Statement> for Block<NORMALIZED> {
    type Arity = Plural;
}
impl<T: IsStatement<NORMALIZED>, const NORMALIZED: bool> HasChild<T, Statement>
    for Block<NORMALIZED>
{
}

impl<T: NameRef> ASTNode for Value<T> {}
impl<T: NameRef> IsExpression for Value<T> {}
impl<T: NameRef> IsStatement for Value<T> {}

impl ASTNode for Literal {}
impl IsExpression for Literal {}
impl IsStatement for Literal {}

impl ASTNode for Tuple {}
impl IsExpression for Tuple {}
impl IsStatement for Tuple {}
impl Family<Expression> for Tuple {
    type Arity = Plural;
}
impl<T: IsExpression> HasChild<T, Expression> for Tuple {}

impl ASTNode for Call {}
impl IsStatement for Call {}
impl IsStatement<true> for Call {}
impl IsExpression for Call {}
impl Family<Lhs> for Call {
    type Arity = Singular;
}
impl Family<Rhs> for Call {
    type Arity = Plural;
}
impl<T: IsExpression> HasChild<T, Lhs> for Call {}
impl<T: IsExpression> HasChild<T, Rhs> for Call {}

impl ASTNode for If {}
impl IsStatement for If {}
impl IsStatement<true> for If {}
impl IsExpression for If {}
impl Family for If {
    type Arity = Plural;
}
impl Family<super::tags::Else> for If {
    type Arity = Optional;
}
impl HasChild<Branch, ()> for If {}
impl HasChild<Otherwise, super::tags::Else> for If {}

impl ASTNode for Branch {}
impl Family<Condition> for Branch {
    type Arity = Singular;
}
impl Family<Statement> for Branch {
    type Arity = Singular;
}
impl<T: IsExpression> HasChild<T, Condition> for Branch {}
impl<T: IsStatement> HasChild<T, Statement> for Branch {}

impl ASTNode for Otherwise {}
impl Family<Statement> for Otherwise {
    type Arity = Singular;
}
impl<T: IsStatement> HasChild<T, Statement> for Otherwise {}

impl ASTNode for Link {}
impl IsStatement for Link {}
impl IsStatement<true> for Link {}
impl Family<Expression> for Link {
    type Arity = Singular;
}
impl<T: IsExpression> HasChild<T, Expression> for Link {}
