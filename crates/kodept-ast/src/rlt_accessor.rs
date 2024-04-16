use std::collections::HashMap;
use std::hash::Hash;

use derive_more::{From, TryInto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::code_point::CodePoint;
use kodept_core::ConvertibleToRef;
use kodept_core::structure::{Located, rlt};

use crate::*;
use crate::graph::NodeId;
use crate::traits::IntoASTFamily;

make_ast_node_adaptor!(ASTFamily, lifetimes: [], NodeId, configs: [
    derive(Hash, PartialEq, Eq, From, Debug, TryInto),
    cfg_attr(feature = "serde", derive(Serialize, Deserialize)),
    cfg_attr(feature = "serde", serde(tag = "owner", content = "id"))
]);

#[derive(Clone, From, TryInto, Debug)]
#[try_into(ref)]
pub enum RLTFamily {
    File(rlt::File),
    Module(rlt::Module),
    Struct(rlt::Struct),
    Enum(rlt::Enum),
    Type(rlt::Type),
    TypeName(rlt::new_types::TypeName),
    TypedParameter(rlt::TypedParameter),
    UntypedParameter(rlt::UntypedParameter),
    Variable(rlt::Variable),
    InitializedVariable(rlt::InitializedVariable),
    BodiedFunction(rlt::BodiedFunction),
    Body(rlt::Body),
    BlockLevel(rlt::BlockLevelNode),
    ExpressionBlock(rlt::ExpressionBlock),
    Operation(rlt::Operation),
    Application(rlt::Application),
    Expression(rlt::Expression),
    Term(rlt::Term),
    Reference(rlt::Reference),
    Literal(rlt::Literal),
    CodeFlow(rlt::CodeFlow),
    If(rlt::IfExpr),
    Elif(rlt::ElifExpr),
    Else(rlt::ElseExpr),
}

#[derive(Default, Debug)]
pub struct RLTAccessor {
    links: HashMap<ASTFamily, RLTFamily>,
}

impl RLTAccessor {
    pub fn access<B>(&self, node: &impl IntoASTFamily) -> Option<&B>
    where
        RLTFamily: ConvertibleToRef<B>,
    {
        self.links
            .get(&node.as_member())
            .and_then(|it| it.try_as_ref())
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

    pub fn save<B>(&mut self, key: impl Into<ASTFamily>, value: &B)
    where
        B: Into<RLTFamily> + Clone,
    {
        self.links.insert(key.into(), value.clone().into());
    }
}

impl<'a> From<&'a RLTFamily> for RLTFamily {
    fn from(value: &'a RLTFamily) -> Self {
        value.clone()
    }
}

impl Located for RLTFamily {
    fn location(&self) -> CodePoint {
        match self {
            RLTFamily::File(x) => x.location(),
            RLTFamily::Module(x) => x.location(),
            RLTFamily::Struct(x) => x.location(),
            RLTFamily::Enum(x) => x.location(),
            RLTFamily::Type(x) => x.location(),
            RLTFamily::TypeName(x) => x.location(),
            RLTFamily::TypedParameter(x) => x.location(),
            RLTFamily::UntypedParameter(x) => x.location(),
            RLTFamily::Variable(x) => x.location(),
            RLTFamily::InitializedVariable(x) => x.location(),
            RLTFamily::BodiedFunction(x) => x.location(),
            RLTFamily::Body(x) => x.location(),
            RLTFamily::BlockLevel(x) => x.location(),
            RLTFamily::ExpressionBlock(x) => x.location(),
            RLTFamily::Operation(x) => x.location(),
            RLTFamily::Application(x) => x.location(),
            RLTFamily::Expression(x) => x.location(),
            RLTFamily::Term(x) => x.location(),
            RLTFamily::Reference(x) => x.location(),
            RLTFamily::Literal(x) => x.location(),
            RLTFamily::CodeFlow(x) => x.location(),
            RLTFamily::If(x) => x.location(),
            RLTFamily::Elif(x) => x.location(),
            RLTFamily::Else(x) => x.location(),
        }
    }
}
