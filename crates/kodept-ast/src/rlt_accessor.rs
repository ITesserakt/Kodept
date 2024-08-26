use crate::graph::GenericNodeKey;
use derive_more::{Constructor, From, TryInto};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::{rlt, Located};
use slotmap::SecondaryMap;

#[derive(Copy, Clone, From, TryInto, Debug)]
pub enum RLTFamily<'r> {
    File(&'r rlt::File),
    Module(&'r rlt::Module),
    Struct(&'r rlt::Struct),
    Enum(&'r rlt::Enum),
    Type(&'r rlt::Type),
    TypeName(&'r rlt::new_types::TypeName),
    TypedParameter(&'r rlt::TypedParameter),
    UntypedParameter(&'r rlt::UntypedParameter),
    Variable(&'r rlt::Variable),
    InitializedVariable(&'r rlt::InitializedVariable),
    BodiedFunction(&'r rlt::BodiedFunction),
    Body(&'r rlt::Body),
    BlockLevel(&'r rlt::BlockLevelNode),
    ExpressionBlock(&'r rlt::ExpressionBlock),
    Operation(&'r rlt::Operation),
    Application(&'r rlt::Application),
    Expression(&'r rlt::Expression),
    Term(&'r rlt::Term),
    Reference(&'r rlt::Reference),
    Contextual(&'r rlt::ContextualReference),
    Literal(&'r rlt::Literal),
    CodeFlow(&'r rlt::CodeFlow),
    If(&'r rlt::IfExpr),
    Elif(&'r rlt::ElifExpr),
    Else(&'r rlt::ElseExpr),
}

#[derive(Constructor, Debug)]
pub struct RLTAccessor<'r> {
    mapping: SecondaryMap<GenericNodeKey, RLTFamily<'r>>,
    root_mapping: Option<RLTFamily<'r>>,
}

impl Located for RLTFamily<'_> {
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
            RLTFamily::Contextual(x) => x.location(),
        }
    }
}
