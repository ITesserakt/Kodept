use crate::v3::tags::{
    Condition, Declaration, Expression, IsDeclaration, IsExpression, IsStatement, Lhs, Rhs,
    Statement,
};
use crate::v3::types::{
    AnonFunction, Block, Branch, Call, ForeignFunction, If, Link, Literal, Module, Otherwise,
    PrimType, Tuple, TypeRef, UserFunction, UserType, Value, ValueCtor, Variable,
};
use crate::{
    Modules, NormalizedBlock, ResolvedType, ResolvedTypeAnnotation, TypeAnnotation, UnresolvedType,
};
use kodept_ast::arity::{Optional, Plural, Singular};
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{Name, RequireProperty};
use kodept_ast::syntax_tree::children::{Family, HasChild};

type Expressions = (
    AnonFunction<TypeAnnotation>,
    AnonFunction<ResolvedTypeAnnotation>,
    Block,
    NormalizedBlock,
    Call,
    If,
    Literal,
    Tuple,
    Value,
);

impl ASTNode for Modules {}
impl Family for Modules {
    type Arity = Plural;
    type Members = (Module,);
}
impl HasChild<Module, ()> for Modules {}

impl ASTNode for Module {}
impl RequireProperty<Name> for Module {}
impl Family<Declaration> for Module {
    type Arity = Plural;
    type Members = (
        PrimType,
        UserType,
        ForeignFunction<UnresolvedType>,
        ForeignFunction<ResolvedType>,
        UserFunction<TypeAnnotation>,
        UserFunction<ResolvedTypeAnnotation>,
    );
}
impl<T: IsDeclaration> HasChild<T, Declaration> for Module {}

impl ASTNode for UserType {}
impl IsDeclaration for UserType {}
impl RequireProperty<Name> for UserType {}
impl Family for UserType {
    type Arity = Plural;
    type Members = (ValueCtor<UnresolvedType>, ValueCtor<ResolvedType>);
}
impl Family<Declaration> for UserType {
    type Arity = Plural;
    type Members = (
        UserFunction<TypeAnnotation>,
        UserFunction<ResolvedTypeAnnotation>,
    );
}
impl<T: TypeRef<true>> HasChild<ValueCtor<T>, ()> for UserType {}
impl<T: TypeRef<false>> HasChild<UserFunction<T>, Declaration> for UserType {}

impl<T: TypeRef<true>> ASTNode for ValueCtor<T> {}

impl ASTNode for PrimType {}
impl IsDeclaration for PrimType {}
impl RequireProperty<Name> for PrimType {}

impl<T: TypeRef<false>> ASTNode for UserFunction<T> {}
impl<T: TypeRef<false>> IsDeclaration for UserFunction<T> {}
impl<T: TypeRef<false>> IsStatement for UserFunction<T> {}
impl<T: TypeRef<false>> IsStatement<true> for UserFunction<T> {}
impl<T: TypeRef<false>> RequireProperty<Name> for UserFunction<T> {}
impl<U: TypeRef<false>> Family for UserFunction<U> {
    type Arity = Singular;
    type Members = (Block, NormalizedBlock);
}
impl<U: TypeRef<false>, const NORMALIZED: bool> HasChild<Block<NORMALIZED>, ()>
    for UserFunction<U>
{
}

impl<T: TypeRef<true>> ASTNode for ForeignFunction<T> {}
impl<T: TypeRef<true>> IsDeclaration for ForeignFunction<T> {}
impl<T: TypeRef<true>> RequireProperty<Name> for ForeignFunction<T> {}

impl<T: TypeRef<false>> ASTNode for AnonFunction<T> {}
impl<T: TypeRef<false>> IsExpression for AnonFunction<T> {}
impl<T: TypeRef<false>> IsStatement for AnonFunction<T> {}
impl<T: TypeRef<false>> Family for AnonFunction<T> {
    type Arity = Singular;
    type Members = (Block, NormalizedBlock);
}
impl<T: TypeRef<false>, const NORMALIZED: bool> HasChild<Block<NORMALIZED>, ()>
    for AnonFunction<T>
{
}

impl<T: TypeRef<false>> ASTNode for Variable<T> {}
impl<T: TypeRef<false>> IsStatement for Variable<T> {}
impl<T: TypeRef<false>> IsStatement<true> for Variable<T> {}
impl<T: TypeRef<false>> Family<Expression> for Variable<T> {
    type Arity = Singular;
    type Members = Expressions;
}
impl<T: IsExpression, U: TypeRef<false>> HasChild<T, Expression> for Variable<U> {}

impl<const NORMALIZED: bool> ASTNode for Block<NORMALIZED> {}
impl<const NORMALIZED: bool> IsStatement<NORMALIZED> for Block<NORMALIZED> {}
impl<const NORMALIZED: bool> IsExpression for Block<NORMALIZED> {}
impl Family<Statement> for Block<false> {
    type Arity = Plural;
    type Members = (
        AnonFunction<TypeAnnotation>,
        AnonFunction<ResolvedTypeAnnotation>,
        Block,
        Call,
        If,
        Link,
        Literal,
        Tuple,
        UserFunction<TypeAnnotation>,
        UserFunction<ResolvedTypeAnnotation>,
        Variable<TypeAnnotation>,
        Variable<ResolvedTypeAnnotation>,
        Value,
    );
}
impl Family<Statement> for Block<true> {
    type Arity = Plural;
    type Members = (
        NormalizedBlock,
        Call,
        If,
        Link,
        UserFunction<TypeAnnotation>,
        UserFunction<ResolvedTypeAnnotation>,
        Variable<TypeAnnotation>,
        Variable<ResolvedTypeAnnotation>,
    );
}
impl<T: IsStatement<false>> HasChild<T, Statement> for Block<false> {}
impl<T: IsStatement<true>> HasChild<T, Statement> for Block<true> {}

impl ASTNode for Value {}
impl IsExpression for Value {}
impl IsStatement for Value {}

impl ASTNode for Literal {}
impl IsExpression for Literal {}
impl IsStatement for Literal {}

impl ASTNode for Tuple {}
impl IsExpression for Tuple {}
impl IsStatement for Tuple {}
impl Family<Expression> for Tuple {
    type Arity = Plural;
    type Members = Expressions;
}
impl<T: IsExpression> HasChild<T, Expression> for Tuple {}

impl ASTNode for Call {}
impl IsStatement for Call {}
impl IsStatement<true> for Call {}
impl IsExpression for Call {}
impl Family<Lhs> for Call {
    type Arity = Singular;
    type Members = Expressions;
}
impl Family<Rhs> for Call {
    type Arity = Plural;
    type Members = Expressions;
}
impl<T: IsExpression> HasChild<T, Lhs> for Call {}
impl<T: IsExpression> HasChild<T, Rhs> for Call {}

impl ASTNode for If {}
impl IsStatement for If {}
impl IsStatement<true> for If {}
impl IsExpression for If {}
impl Family for If {
    type Arity = Plural;
    type Members = (Branch,);
}
impl Family<super::tags::Else> for If {
    type Arity = Optional;
    type Members = (Otherwise,);
}
impl HasChild<Branch, ()> for If {}
impl HasChild<Otherwise, super::tags::Else> for If {}

impl ASTNode for Branch {}
impl Family<Condition> for Branch {
    type Arity = Singular;
    type Members = Expressions;
}
impl Family<Expression> for Branch {
    type Arity = Singular;
    type Members = (Block, NormalizedBlock);
}
impl<T: IsExpression> HasChild<T, Condition> for Branch {}
impl<const NORMALIZED: bool> HasChild<Block<NORMALIZED>, Expression> for Branch {}

impl ASTNode for Otherwise {}
impl Family<Expression> for Otherwise {
    type Arity = Singular;
    type Members = (Block, NormalizedBlock);
}
impl<const NORMALIZED: bool> HasChild<Block<NORMALIZED>, Expression> for Otherwise {}

impl ASTNode for Link {}
impl IsStatement for Link {}
impl IsStatement<true> for Link {}
impl Family<Expression> for Link {
    type Arity = Singular;
    type Members = Expressions;
}
impl<T: IsExpression> HasChild<T, Expression> for Link {}

#[allow(unsafe_code)]
mod transmutes {
    use crate::{
        AnonFunction, Block, ForeignFunction, NormalizedBlock, ResolvedType,
        ResolvedTypeAnnotation, TypeAnnotation, UnresolvedType, UserFunction, ValueCtor, Variable,
    };
    use kodept_ast::experimental::TransmuteInto;

    unsafe impl TransmuteInto<NormalizedBlock> for Block {}
    unsafe impl TransmuteInto<UserFunction<ResolvedTypeAnnotation>> for UserFunction<TypeAnnotation> {}
    unsafe impl TransmuteInto<ForeignFunction<ResolvedType>> for ForeignFunction<UnresolvedType> {}
    unsafe impl TransmuteInto<AnonFunction<ResolvedTypeAnnotation>> for AnonFunction<TypeAnnotation> {}
    unsafe impl TransmuteInto<Variable<ResolvedTypeAnnotation>> for Variable<TypeAnnotation> {}
    unsafe impl TransmuteInto<ValueCtor<ResolvedType>> for ValueCtor<UnresolvedType> {}
}
