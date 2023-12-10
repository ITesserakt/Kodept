use crate::block_level::{InitializedVariable, Variable};
use crate::node_id::NodeId;
use crate::traits::Identifiable;
use crate::*;
use derive_more::{From, TryInto};
use kodept_core::structure::rlt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use std::collections::HashMap;
use std::hash::Hash;

make_ast_node_adaptor!(ASTFamily, lifetimes: [], NodeId, configs: [
    derive(Hash, PartialEq, Eq, From, Debug),
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
    pub fn access<A, B>(&self, node: &A) -> Option<&'n B>
    where
        NodeId<A>: Into<ASTFamily>,
        A: Identifiable + 'static,
        &'n B: TryFrom<RLTFamily<'n>> + 'n,
    {
        self.links
            .get(&node.get_id().clone().into())
            .and_then(|it| it.clone().try_into().ok())
    }

    pub fn access_unknown<A>(&self, node: &A) -> Option<&RLTFamily>
    where
        NodeId<A>: Into<ASTFamily>,
        A: Identifiable + 'static,
    {
        self.links.get(&node.get_id().clone().into())
    }

    pub fn save_existing<A, B>(&mut self, new: &A, existing: &B)
    where
        B: Identifiable + 'static,
        NodeId<B>: Into<ASTFamily>,
        A: Identifiable + 'static,
        NodeId<A>: Into<ASTFamily>,
    {
        match self.links.get(&existing.get_id().clone().into()) {
            None => None,
            Some(x) => self.links.insert(new.get_id().clone().into(), x.clone()),
        };
    }

    pub fn keys(&self) -> Vec<&ASTFamily> {
        self.links.keys().collect()
    }

    pub fn save<A, B>(&mut self, key: NodeId<A>, value: B)
    where
        B: Into<RLTFamily<'n>>,
        NodeId<A>: Into<ASTFamily>,
    {
        self.links.insert(key.into(), value.into());
    }
}

impl<'a, 'b> From<&'a RLTFamily<'b>> for RLTFamily<'b> {
    fn from(value: &'a RLTFamily<'b>) -> Self {
        value.clone()
    }
}
