use crate::v3::tags::{
    Condition, Declaration, Expression, IsDeclaration, IsExpression, IsStatement, Lhs, Rhs,
    Statement,
};
use crate::v3::types::{
    AnonFunction, Block, Branch, Call, ForeignFunction, If, Literal, Module, NameRef, Otherwise,
    PrimType, Tuple, TypeCtor, TypeRef, UserFunction, UserType, Value, Variable,
};
use bevy_ecs::prelude::Name;
use kodept_ast::arity::{Optional, Plural, Singular};
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{HasProperty, RequireProperty};
use kodept_ast::syntax_tree::children::HasChild;

impl ASTNode for Module {}
impl RequireProperty<Name> for Module {}
impl<T: IsDeclaration> HasChild<T, Declaration> for Module {
    type Arity = Plural;
}

impl ASTNode for UserType {}
impl IsDeclaration for UserType {}
impl HasProperty<Name> for UserType {}
impl<T: TypeRef<true>> HasChild<TypeCtor<T>, ()> for UserType {
    type Arity = Plural;
}
impl<T: TypeRef<false>> HasChild<UserFunction<T>, Declaration> for UserType {
    type Arity = Plural;
}

impl<T: TypeRef<true>> ASTNode for TypeCtor<T> {}

impl ASTNode for PrimType {}
impl IsDeclaration for PrimType {}

impl<T: TypeRef<false>> ASTNode for UserFunction<T> {}
impl<T: TypeRef<false>> IsDeclaration for UserFunction<T> {}
impl<T: TypeRef<false>> IsStatement for UserFunction<T> {}
impl<T: TypeRef<false>> RequireProperty<Name> for UserFunction<T> {}
impl<U: TypeRef<false>> HasChild<Block, ()> for UserFunction<U> {
    type Arity = Singular;
}

impl<T: TypeRef<true>> ASTNode for ForeignFunction<T> {}
impl<T: TypeRef<true>> IsDeclaration for ForeignFunction<T> {}
impl<T: TypeRef<true>> RequireProperty<Name> for ForeignFunction<T> {}

impl<T: TypeRef<false>> ASTNode for AnonFunction<T> {}
impl<T: TypeRef<false>> IsExpression for AnonFunction<T> {}
impl<T: TypeRef<false>> IsStatement for AnonFunction<T> {}
impl<T: TypeRef<false>> HasChild<Block, ()> for AnonFunction<T> {
    type Arity = Singular;
}

impl<T: TypeRef<false>> ASTNode for Variable<T> {}
impl<T: TypeRef<false>> IsStatement for Variable<T> {}
impl<T: TypeRef<false>> RequireProperty<Name> for Variable<T> {}
impl<T: IsExpression, U: TypeRef<false>> HasChild<T, Expression> for Variable<U> {
    type Arity = Singular;
}

impl ASTNode for Block {}
impl IsStatement for Block {}
impl IsExpression for Block {}
impl<T: IsStatement> HasChild<T, Statement> for Block {
    type Arity = Plural;
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
impl<T: IsExpression> HasChild<T, Expression> for Tuple {
    type Arity = Plural;
}

impl ASTNode for Call {}
impl IsStatement for Call {}
impl IsExpression for Call {}
impl<T: IsExpression> HasChild<T, Lhs> for Call {
    type Arity = Singular;
}
impl<T: IsExpression> HasChild<T, Rhs> for Call {
    type Arity = Plural;
}

impl ASTNode for If {}
impl IsStatement for If {}
impl IsExpression for If {}
impl HasChild<Branch, ()> for If {
    type Arity = Plural;
}
impl HasChild<Otherwise, ()> for If {
    type Arity = Optional;
}

impl ASTNode for Branch {}
impl<T: IsExpression> HasChild<T, Condition> for Branch {
    type Arity = Singular;
}
impl<T: IsStatement> HasChild<T, Statement> for Branch {
    type Arity = Singular;
}

impl ASTNode for Otherwise {}
impl<T: IsStatement> HasChild<T, Statement> for Otherwise {
    type Arity = Singular;
}
