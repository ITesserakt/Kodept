use std::fmt::Debug;

use crate::graph::node_id::AnyNodeId;
use crate::graph::node_props::{ConversionError, Node, SubEnum};
use crate::graph::Identifiable;
use crate::*;
use derive_more::{Display, From, TryInto};
use kodept_core::static_assert_size;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, IntoStaticStr, VariantArray, VariantNames};

#[derive(Debug, PartialEq, From, TryInto, EnumDiscriminants, IntoStaticStr, VariantNames)]
#[strum_discriminants(derive(VariantArray, Display))]
#[strum_discriminants(cfg_attr(feature = "serde", derive(Serialize, Deserialize)))]
#[strum_discriminants(name(AnyNodeD))]
#[try_into(owned, ref, ref_mut)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AnyNode {
    FileDecl(FileDecl),
    ModDecl(ModDecl),
    StructDecl(StructDecl),
    EnumDecl(EnumDecl),
    TyParam(TyParam),
    NonTyParam(NonTyParam),
    TyName(TyName),
    VarDecl(VarDecl),
    InitVar(InitVar),
    BodyFnDecl(BodyFnDecl),
    Exprs(Exprs),
    Appl(Appl),
    Lambda(Lambda),
    Ref(Ref),
    Acc(Acc),
    NumLit(NumLit),
    CharLit(CharLit),
    StrLit(StrLit),
    TupleLit(TupleLit),
    IfExpr(IfExpr),
    ElifExpr(ElifExpr),
    ElseExpr(ElseExpr),
    BinExpr(BinExpr),
    UnExpr(UnExpr),
    AbstFnDecl(AbstFnDecl),
    ProdTy(ProdTy),
}

// It's important to support the size of AnyNode less than 64 to fit into a cache line
static_assert_size!(AnyNode, 56);
static_assert_size!(AnyNodeD, 1);

macro_rules! folding {
    ($this:expr; $bind:ident => $usage:expr) => {
        match $this {
            AnyNode::FileDecl($bind) => $usage,
            AnyNode::ModDecl($bind) => $usage,
            AnyNode::StructDecl($bind) => $usage,
            AnyNode::EnumDecl($bind) => $usage,
            AnyNode::TyParam($bind) => $usage,
            AnyNode::NonTyParam($bind) => $usage,
            AnyNode::TyName($bind) => $usage,
            AnyNode::VarDecl($bind) => $usage,
            AnyNode::InitVar($bind) => $usage,
            AnyNode::BodyFnDecl($bind) => $usage,
            AnyNode::Exprs($bind) => $usage,
            AnyNode::Appl($bind) => $usage,
            AnyNode::Lambda($bind) => $usage,
            AnyNode::Ref($bind) => $usage,
            AnyNode::Acc($bind) => $usage,
            AnyNode::NumLit($bind) => $usage,
            AnyNode::CharLit($bind) => $usage,
            AnyNode::StrLit($bind) => $usage,
            AnyNode::TupleLit($bind) => $usage,
            AnyNode::IfExpr($bind) => $usage,
            AnyNode::ElifExpr($bind) => $usage,
            AnyNode::ElseExpr($bind) => $usage,
            AnyNode::BinExpr($bind) => $usage,
            AnyNode::UnExpr($bind) => $usage,
            AnyNode::AbstFnDecl($bind) => $usage,
            AnyNode::ProdTy($bind) => $usage,
        }
    };
}

impl SubEnum for AnyNode {
    const VARIANTS: &'static [AnyNodeD] = AnyNodeD::VARIANTS;
}

impl Identifiable for AnyNode {
    #[inline]
    fn get_id(&self) -> AnyNodeId {
        folding!(self; x => x.get_id().widen())
    }

    #[inline]
    fn set_id(&self, value: AnyNodeId) {
        folding!(self; x => x.set_id(value.narrow()));
    }
}

impl Node for AnyNode {
    fn erase(self) -> AnyNode {
        self
    }

    fn describe(&self) -> AnyNodeD {
        AnyNode::describe(self)
    }

    fn try_from_ref(value: &AnyNode) -> Result<&Self, ConversionError> {
        Ok(value)
    }

    fn try_from_mut(value: &mut AnyNode) -> Result<&mut Self, ConversionError> {
        Ok(value)
    }
}

impl AnyNode {
    #[inline]
    pub fn describe(&self) -> AnyNodeD {
        self.into()
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.into()
    }
}
