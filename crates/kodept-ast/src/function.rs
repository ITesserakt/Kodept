use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::node_group;

use crate::node_id::NodeId;
use crate::traits::{IdProducer, Identifiable, Instantiable, IntoAst, Linker};
use crate::{impl_identifiable, Body, Parameter, Type};

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BodiedFunctionDeclaration {
    id: NodeId<Self>,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Body,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct AbstractFunctionDeclaration {
    id: NodeId<Self>,
    pub name: String,
    pub return_type: Option<Type>,
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum FunctionDeclaration {
    Abstract(AbstractFunctionDeclaration),
    Bodied(BodiedFunctionDeclaration),
}

node_group!(family: FunctionDeclaration, nodes: [
    FunctionDeclaration, BodiedFunctionDeclaration, AbstractFunctionDeclaration
]);
node_group!(family: BodiedFunctionDeclaration, nodes: [BodiedFunctionDeclaration, Body]);
node_group!(family: AbstractFunctionDeclaration, nodes: [AbstractFunctionDeclaration]);
impl_identifiable! {
    BodiedFunctionDeclaration,
    AbstractFunctionDeclaration
}

impl Identifiable for FunctionDeclaration {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            FunctionDeclaration::Abstract(x) => x.get_id().cast(),
            FunctionDeclaration::Bodied(x) => x.get_id().cast(),
        }
    }
}

impl IntoAst for rlt::BodiedFunction {
    type Output = BodiedFunctionDeclaration;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = BodiedFunctionDeclaration {
            id: context.next_id(),
            name: context.get_chunk_located(&self.id).to_string(),
            parameters: self
                .params
                .as_ref()
                .map_or(vec![], |it| it.inner.iter().map(|it| todo!()).collect()),
            return_type: self.return_type.as_ref().map(|it| todo!()),
            body: self.body.construct(context),
        };
        context.link(node, self)
    }
}

impl Instantiable for BodiedFunctionDeclaration {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            id: context.next_id(),
            name: self.name.clone(),
            parameters: self.parameters.iter().map(|it| todo!()).collect(),
            return_type: self.return_type.as_ref().map(|it| todo!()),
            body: self.body.new_instance(context),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for AbstractFunctionDeclaration {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            id: context.next_id(),
            name: self.name.clone(),
            return_type: self.return_type.as_ref().map(|it| todo!()),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for FunctionDeclaration {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            FunctionDeclaration::Abstract(x) => x.new_instance(context).into(),
            FunctionDeclaration::Bodied(x) => x.new_instance(context).into(),
        }
    }
}
