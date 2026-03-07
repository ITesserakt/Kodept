use crate::v3::tags::{
    Condition, Declaration, Expression, IsDeclaration, IsExpression, IsStatement, Lhs, Rhs,
    Statement,
};
use crate::v3::types::{
    AnonFunction, Block, Branch, Call, ForeignFunction, If, Link, Literal, Module, Otherwise,
    PrimType, Tuple, UserFunction, UserType, Value, ValueCtor, Variable,
};
use crate::{Modules, NamedParams, NormalizedBlock, Param, Params};
use kodept_ast::arity::{Optional, Plural, Singular};
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{Name, RequireProperty};
use kodept_ast::syntax_tree::children::{Family, HasChild};

type Expressions = (
    AnonFunction,
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
    type Members = (PrimType, UserType, ForeignFunction, UserFunction);
}
impl<T: IsDeclaration> HasChild<T, Declaration> for Module {}

impl ASTNode for UserType {}
impl IsDeclaration for UserType {}
impl RequireProperty<Name> for UserType {}
impl Family for UserType {
    type Arity = Plural;
    type Members = (ValueCtor,);
}
impl Family<Declaration> for UserType {
    type Arity = Plural;
    type Members = (UserFunction,);
}
impl HasChild<ValueCtor, ()> for UserType {}
impl HasChild<UserFunction, Declaration> for UserType {}

impl ASTNode for ValueCtor {}
impl Family<Params> for ValueCtor {
    type Arity = Plural;
    type Members = (Param,);
}
impl HasChild<Param, Params> for ValueCtor {}

impl ASTNode for PrimType {}
impl IsDeclaration for PrimType {}
impl RequireProperty<Name> for PrimType {}

impl ASTNode for UserFunction {}
impl IsDeclaration for UserFunction {}
impl IsStatement for UserFunction {}
impl IsStatement<true> for UserFunction {}
impl RequireProperty<Name> for UserFunction {}
impl Family for UserFunction {
    type Arity = Singular;
    type Members = (Block, NormalizedBlock);
}
impl Family<Params> for UserFunction {
    type Arity = Plural;
    type Members = (Param,);
}
impl Family<NamedParams> for UserFunction {
    type Arity = Plural;
    type Members = (Param,);
}
impl HasChild<Param, Params> for UserFunction {}
impl HasChild<Param, NamedParams> for UserFunction {}
impl<const NORMALIZED: bool> HasChild<Block<NORMALIZED>, ()> for UserFunction {}

impl ASTNode for ForeignFunction {}
impl IsDeclaration for ForeignFunction {}
impl RequireProperty<Name> for ForeignFunction {}
impl Family<Params> for ForeignFunction {
    type Arity = Plural;
    type Members = (Param,);
}
impl HasChild<Param, Params> for ForeignFunction {}

impl ASTNode for AnonFunction {}
impl IsExpression for AnonFunction {}
impl IsStatement for AnonFunction {}
impl Family for AnonFunction {
    type Arity = Singular;
    type Members = (Block, NormalizedBlock);
}
impl Family<Params> for AnonFunction {
    type Arity = Plural;
    type Members = (Param,);
}
impl<const NORMALIZED: bool> HasChild<Block<NORMALIZED>, ()> for AnonFunction {}
impl HasChild<Param, Params> for AnonFunction {}

impl ASTNode for Variable {}
impl IsStatement for Variable {}
impl IsStatement<true> for Variable {}
impl Family<Expression> for Variable {
    type Arity = Singular;
    type Members = Expressions;
}
impl<T: IsExpression> HasChild<T, Expression> for Variable {}

impl<const NORMALIZED: bool> ASTNode for Block<NORMALIZED> {}
impl<const NORMALIZED: bool> IsStatement<NORMALIZED> for Block<NORMALIZED> {}
impl<const NORMALIZED: bool> IsExpression for Block<NORMALIZED> {}
impl Family<Statement> for Block<false> {
    type Arity = Plural;
    type Members = (
        AnonFunction,
        Block,
        Call,
        If,
        Link,
        Literal,
        Tuple,
        UserFunction,
        Variable,
        // Value,
    );
}
impl Family<Statement> for Block<true> {
    type Arity = Plural;
    type Members = (NormalizedBlock, Call, If, Link, UserFunction, Variable);
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

impl ASTNode for Param {}
impl RequireProperty<Name> for Param {}

#[allow(unsafe_code)]
mod transmutes {
    use crate::{Block, NormalizedBlock};
    use kodept_ast::experimental::TransmuteInto;

    unsafe impl TransmuteInto<NormalizedBlock> for Block {}
}
