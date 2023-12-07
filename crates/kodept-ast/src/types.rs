use crate::impl_identifiable;
use crate::node_id::NodeId;
use crate::traits::{IdProducer, Identifiable, Instantiable, IntoAst, Linker};
use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::node_group;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TypeName {
    pub name: String,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ProdType {
    pub types: Vec<Type>,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SumType {
    pub types: Vec<Type>,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Type {
    Reference(TypeName),
    Tuple(ProdType),
    Union(SumType),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TypedParameter {
    pub name: String,
    pub parameter_type: Type,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct UntypedParameter {
    pub name: String,
    id: NodeId<Self>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Parameter {
    Typed(TypedParameter),
    Untyped(UntypedParameter),
}

#[cfg(feature = "size-of")]
impl SizeOf for Type {
    fn size_of_children(&self, context: &mut size_of::Context) {
        match self {
            Type::Reference(x) => x.size_of_children(context),
            Type::Tuple(x) => x.size_of_children(context),
            Type::Union(x) => x.size_of_children(context),
        }
    }
}

node_group! {
    family: Type,
    nodes: [Type, TypeName, ProdType, SumType]
}

node_group! {
    family: TypedParameter,
    nodes: [TypedParameter, Type]
}

impl_identifiable! {TypeName, ProdType, SumType, TypedParameter, UntypedParameter}

impl IntoAst for rlt::Type {
    type Output = Type;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        match self {
            rlt::Type::Reference(x) => x.construct(context).into(),
            rlt::Type::Tuple(x) => ProdType {
                types: x.inner.iter().map(|it| it.construct(context)).collect(),
                id: context.next_id(),
            }
            .into(),
            rlt::Type::Union(x) => SumType {
                types: x.inner.iter().map(|it| it.construct(context)).collect(),
                id: context.next_id(),
            }
            .into(),
        }
    }
}

impl IntoAst for rlt::Parameter {
    type Output = Parameter;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        match self {
            rlt::Parameter::Typed(x) => Parameter::Typed(x.construct(context)),
            rlt::Parameter::Untyped(x) => Parameter::Untyped(x.construct(context)),
        }
    }
}

impl IntoAst for rlt::new_types::TypeName {
    type Output = TypeName;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = TypeName {
            name: context.get_chunk_located(self).to_string(),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::TypedParameter {
    type Output = TypedParameter;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = TypedParameter {
            name: context.get_chunk_located(&self.id).to_string(),
            parameter_type: self.parameter_type.construct(context),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::UntypedParameter {
    type Output = UntypedParameter;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = UntypedParameter {
            name: context.get_chunk_located(&self.id).to_string(),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl Instantiable for TypeName {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            name: self.name.clone(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for ProdType {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            types: self
                .types
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for SumType {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            types: self
                .types
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Identifiable for Type {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            Type::Reference(x) => x.get_id().cast(),
            Type::Tuple(x) => x.get_id().cast(),
            Type::Union(x) => x.get_id().cast(),
        }
    }
}

impl Instantiable for TypedParameter {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            name: self.name.clone(),
            parameter_type: self.parameter_type.new_instance(context),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for UntypedParameter {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            name: self.name.clone(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for Parameter {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            Parameter::Typed(x) => Parameter::Typed(x.new_instance(context)),
            Parameter::Untyped(x) => Parameter::Untyped(x.new_instance(context)),
        }
    }
}

impl Instantiable for Type {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            Type::Reference(x) => x.new_instance(context).into(),
            Type::Tuple(x) => x.new_instance(context).into(),
            Type::Union(x) => x.new_instance(context).into(),
        }
    }
}
