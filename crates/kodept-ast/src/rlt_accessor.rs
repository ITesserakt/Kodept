use std::collections::HashMap;
use std::hash::Hash;

use derive_more::{From, TryInto};
use kodept_core::ConvertibleTo;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::structure::rlt;

use crate::graph::NodeId;
use crate::traits::IntoASTFamily;
use crate::*;

make_ast_node_adaptor!(ASTFamily, lifetimes: [], NodeId, configs: [
    derive(Hash, PartialEq, Eq, From, Debug, TryInto),
    cfg_attr(feature = "size-of", derive(SizeOf)),
    cfg_attr(feature = "serde", derive(Serialize, Deserialize)),
    cfg_attr(feature = "serde", serde(tag = "owner", content = "id"))
]);

#[derive(Clone, From, TryInto, Debug)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum RLTFamily<'n> {
    File(&'n rlt::File),
    Module(&'n rlt::Module),
    Struct(&'n rlt::Struct),
    Enum(&'n rlt::Enum),
    Type(&'n rlt::Type),
    TypeName(&'n rlt::new_types::TypeName),
    TypedParameter(&'n rlt::TypedParameter),
    UntypedParameter(&'n rlt::UntypedParameter),
    Variable(&'n rlt::Variable),
    InitializedVariable(&'n rlt::InitializedVariable),
    BodiedFunction(&'n rlt::BodiedFunction),
    Body(&'n rlt::Body),
    BlockLevel(&'n rlt::BlockLevelNode),
    ExpressionBlock(&'n rlt::ExpressionBlock),
    Operation(&'n rlt::Operation),
    Application(&'n rlt::Application),
    Expression(&'n rlt::Expression),
    Term(&'n rlt::Term),
    Reference(&'n rlt::Reference),
    Literal(&'n rlt::Literal),
    CodeFlow(&'n rlt::CodeFlow),
    If(&'n rlt::IfExpr),
    Elif(&'n rlt::ElifExpr),
    Else(&'n rlt::ElseExpr),
}

#[derive(Default, Debug)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct RLTAccessor<'n> {
    links: HashMap<ASTFamily, RLTFamily<'n>>,
}

impl<'n> RLTAccessor<'n> {
    pub fn access<B: 'n>(&self, node: &impl IntoASTFamily) -> Option<&B>
    where
        RLTFamily<'n>: ConvertibleTo<&'n B>,
    {
        self.links
            .get(&node.as_member())
            .and_then(|it| it.clone().try_as())
    }

    pub fn access_unknown(&self, node: &impl IntoASTFamily) -> Option<&RLTFamily> {
        self.links.get(&node.as_member())
    }

    pub fn save_existing(&mut self, new: &impl IntoASTFamily, existing: &impl IntoASTFamily) {
        match self.links.get(&existing.as_member()) {
            None => None,
            Some(x) => self.links.insert(new.as_member(), x.clone()),
        };
    }

    pub fn keys(&self) -> Vec<&ASTFamily> {
        self.links.keys().collect()
    }

    pub fn save<B>(&mut self, key: impl Into<ASTFamily>, value: B)
    where
        B: Into<RLTFamily<'n>>,
    {
        self.links.insert(key.into(), value.into());
    }
}

impl<'a, 'b> From<&'a RLTFamily<'b>> for RLTFamily<'b> {
    fn from(value: &'a RLTFamily<'b>) -> Self {
        value.clone()
    }
}
