#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::{Identity, SubSyntaxTree};
use crate::interning::SharedStr;
use crate::traits::{AsEnum, PopulateTree};
use crate::{node, node_sub_enum, BodyFnDecl, EnumDecl, StructDecl};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum Type {
        TyName(TyName),
        Tuple(ProdTy)
    }
}

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum Param {
        Ty(TyParam),
        NonTy(NonTyParam)
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TyName {
        pub name: SharedStr,;;
        parent is [TyParam, BodyFnDecl, EnumDecl]
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ProdTy {;
        pub types: Vec<Type>,;
        parent is [TyParam, BodyFnDecl]
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct TyParam {
        pub name: SharedStr,;
        pub parameter_type: Identity<Type>,;
        parent is [StructDecl, BodyFnDecl]
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct NonTyParam {
        pub name: SharedStr,;
    }
}

impl Param {
    pub fn name(&self) -> &str {
        match self.as_enum() {
            ParamEnum::Ty(x) => &x.name,
            ParamEnum::NonTy(x) => &x.name,
        }
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::new_types::TypeName {
    type Root = TyName;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        let node = TyName::uninit(context.get_chunk_located(self)).with_rlt(self);

        SubSyntaxTree::new(node)
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::TypedParameter {
    type Root = TyParam;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        let node = TyParam::uninit(context.get_chunk_located(&self.id)).with_rlt(self);
        SubSyntaxTree::new(node).with_children_from([&self.parameter_type], context)
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::Type {
    type Root = Type;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        match self {
            rlt::Type::Reference(x) => x.convert(context).cast(),
            rlt::Type::Tuple(x) => SubSyntaxTree::new(ProdTy::uninit().with_rlt(self))
                .with_children_from(x.inner.iter().as_slice(), context)
                .cast(),
        }
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::UntypedParameter {
    type Root = NonTyParam;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        SubSyntaxTree::new(
            NonTyParam::uninit(context.get_chunk_located(&self.id)).with_rlt(self),
        )
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::Parameter {
    type Root = Param;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        match self {
            rlt::Parameter::Typed(x) => x.convert(context).cast(),
            rlt::Parameter::Untyped(x) => x.convert(context).cast(),
        }
    }
}
