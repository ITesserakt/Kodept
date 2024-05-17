use derive_more::{From, TryInto};
use slotmap::SecondaryMap;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::{rlt, Located};
use kodept_core::ConvertibleToRef;

use crate::graph::{GenericASTNode, GenericNodeKey};
use crate::traits::Identifiable;

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
    Contextual(rlt::ContextualReference),
    Literal(rlt::Literal),
    CodeFlow(rlt::CodeFlow),
    If(rlt::IfExpr),
    Elif(rlt::ElifExpr),
    Else(rlt::ElseExpr),
}

#[derive(Debug, Default)]
pub struct RLTAccessor {
    links: SecondaryMap<GenericNodeKey, RLTFamily>,
}

impl RLTAccessor {
    pub fn access<A, B>(&self, node: &A) -> Option<&B>
    where
        A: Identifiable + Into<GenericASTNode>,
        RLTFamily: ConvertibleToRef<B>,
    {
        self.links
            .get(node.get_id().widen().into())
            .and_then(|it| it.try_as_ref())
    }

    pub fn access_unknown<A>(&self, node: &A) -> Option<&RLTFamily>
    where
        A: Identifiable + Into<GenericASTNode>,
    {
        self.links.get(node.get_id().widen().into())
    }

    pub fn save_existing<A, B>(&mut self, new: &A, existing: &B)
    where
        A: Identifiable + Into<GenericASTNode>,
        B: Identifiable + Into<GenericASTNode>,
    {
        match self.links.get(existing.get_id().widen().into()) {
            None => None,
            Some(x) => self.links.insert(new.get_id().widen().into(), x.clone()),
        };
    }

    pub fn save<A, B>(&mut self, key: &A, value: &B)
    where
        B: Into<RLTFamily> + Clone,
        A: Identifiable + Into<GenericASTNode>
    {
        self.links.insert(key.get_id().widen().into(), value.clone().into());
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
            RLTFamily::Contextual(x) => x.location()
        }
    }
}
