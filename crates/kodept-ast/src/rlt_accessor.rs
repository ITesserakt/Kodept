use crate::graph::{AnyNode, GenericNodeKey, NodeId};
use derive_more::{From, TryInto};
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

#[derive(Debug)]
pub struct RLTAccessor<'r> {
    mapping: SecondaryMap<GenericNodeKey, RLTFamily<'r>>,
    root_mapping: Option<RLTFamily<'r>>,
}

impl<'r> RLTAccessor<'r> {
    pub(crate) fn new(
        mapping: SecondaryMap<GenericNodeKey, RLTFamily<'r>>,
        root_mapping: Option<RLTFamily<'r>>,
    ) -> Self {
        Self {
            mapping,
            root_mapping,
        }
    }

    pub fn get_unknown<T>(&self, id: NodeId<T>) -> Option<RLTFamily<'r>>
    where
        AnyNode: TryFrom<T>,
    {
        match id {
            NodeId::Root => self.root_mapping,
            NodeId::Key(id) => self.mapping.get(id.coerce()).copied(),
        }
    }

    pub fn get<T, R>(&self, id: NodeId<T>) -> Option<&R>
    where
        for<'a> RLTFamily<'a>: TryInto<&'a R>,
        AnyNode: TryFrom<T>,
    {
        match self.get_unknown(id) {
            None => None,
            Some(x) => x.try_into().ok(),
        }
    }

    pub fn set<T, R>(&mut self, id: NodeId<T>, value: R)
    where
        R: Into<RLTFamily<'r>>,
        AnyNode: From<T>
    {
        let value = value.into();
        match id {
            NodeId::Root => {
                self.root_mapping = Some(value)
            }
            NodeId::Key(_) => {
                self.mapping.insert(id.as_key().unwrap(), value);
            }
        };
    }

    pub(crate) fn append(&mut self, other: RLTAccessor<'r>) {
        if let Some(x) = other.root_mapping {
            self.root_mapping = Some(x)
        }
        for (k, v) in other.mapping {
            self.mapping.insert(k, v);
        }
    }
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
