use bevy_ecs::entity::EntityHash;
use bevy_ecs::prelude::{Entity, Resource};
use dashmap::DashMap;
use derive_more::{From, TryInto};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_core::Freeze;
use kodept_rlt::prelude::RLT;
use kodept_rlt::{new_types, prelude as rlt};
use std::marker::PhantomPinned;
use std::pin::Pin;
use crate::node_id::Erase;

#[derive(Debug, Copy, Clone, PartialEq, TryInto, From)]
pub enum SyntaxVariant<'r> {
    File(&'r rlt::File),
    Module(&'r rlt::Module),
    Struct(&'r rlt::Struct),
    Enum(&'r rlt::Enum),
    Type(&'r rlt::Type),
    TypeName(&'r new_types::TypeName),
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
    Tuple(&'r rlt::Tuple),
    Lambda(&'r rlt::Lambda),
}

#[derive(Debug)]
struct PinnedRLT {
    inner: Freeze<RLT>,
    _phantom: PhantomPinned,
}

#[derive(Debug, Resource)]
pub struct SyntaxResolver {
    tree: Pin<Box<PinnedRLT>>,
    mapping: DashMap<Entity, SyntaxVariant<'static>, EntityHash>,
}

pub enum LookupError {
    NotFound,
    WrongType,
}

impl SyntaxResolver {
    pub fn empty(tree: RLT) -> Self {
        Self {
            tree: Box::pin(PinnedRLT {
                inner: Freeze::new(tree),
                _phantom: Default::default(),
            }),
            mapping: Default::default(),
        }
    }

    pub fn root(&self) -> &rlt::File {
        &self.tree.inner.0
    }

    // TODO: probably unsound code
    #[allow(unsafe_code)]
    pub fn insert<'r, U>(&'r self, id: impl Erase<Entity>, node: U)
    where
        U: Into<SyntaxVariant<'r>>,
    {
        let variant = node.into();
        // SAFETY: lifetimes of node and self are equal and produced reference won't be used in 'static contexts
        let reborrow =
            unsafe { std::mem::transmute::<SyntaxVariant<'r>, SyntaxVariant<'static>>(variant) };
        self.mapping.insert(id.erase(), reborrow);
    }

    pub fn get_unknown(&self, id: impl Erase) -> SyntaxVariant {
        self.try_get_unknown(id)
            .expect("Cannot get linked RLT node")
    }

    pub fn get_location(&self, id: impl Erase) -> CodePoint {
        let id = id.erase().into();
        self.mapping
            .get(&id)
            .expect("Cannot get linked RLT node")
            .location()
    }

    pub fn try_get_unknown(&self, id: impl Erase) -> Option<SyntaxVariant> {
        let id = id.erase().into();
        let reference = self.mapping.get(&id)?;
        Some(*reference.value())
    }

    pub fn try_get<'r, U>(&'r self, id: impl Erase) -> Result<&'r U, LookupError>
    where
        &'r U: TryFrom<SyntaxVariant<'r>>,
    {
        let variant = self.try_get_unknown(id).ok_or(LookupError::NotFound)?;
        variant.try_into().map_err(|_| LookupError::WrongType)
    }
}

impl Located for SyntaxVariant<'_> {
    fn location(&self) -> CodePoint {
        match self {
            SyntaxVariant::File(x) => x.location(),
            SyntaxVariant::Module(x) => x.location(),
            SyntaxVariant::Struct(x) => x.location(),
            SyntaxVariant::Enum(x) => x.location(),
            SyntaxVariant::Type(x) => x.location(),
            SyntaxVariant::TypeName(x) => x.location(),
            SyntaxVariant::TypedParameter(x) => x.location(),
            SyntaxVariant::UntypedParameter(x) => x.location(),
            SyntaxVariant::Variable(x) => x.location(),
            SyntaxVariant::InitializedVariable(x) => x.location(),
            SyntaxVariant::BodiedFunction(x) => x.location(),
            SyntaxVariant::Body(x) => x.location(),
            SyntaxVariant::BlockLevel(x) => x.location(),
            SyntaxVariant::ExpressionBlock(x) => x.location(),
            SyntaxVariant::Operation(x) => x.location(),
            SyntaxVariant::Application(x) => x.location(),
            SyntaxVariant::Expression(x) => x.location(),
            SyntaxVariant::Term(x) => x.location(),
            SyntaxVariant::Reference(x) => x.location(),
            SyntaxVariant::Contextual(x) => x.location(),
            SyntaxVariant::Literal(x) => x.location(),
            SyntaxVariant::CodeFlow(x) => x.location(),
            SyntaxVariant::If(x) => x.location(),
            SyntaxVariant::Elif(x) => x.location(),
            SyntaxVariant::Else(x) => x.location(),
            SyntaxVariant::Tuple(x) => x.location(),
            SyntaxVariant::Lambda(x) => x.location(),
        }
    }
}
