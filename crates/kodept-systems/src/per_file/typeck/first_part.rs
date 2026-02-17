use crate::per_file::symbols::SymbolKind;
use crate::source::collection::Reporter;
use kodept_ast::prelude::{NarrowHierarchicalQuery, NodeId};
use kodept_ast::properties::{NodeProperty, RequireProperty, SourceSpan};
use kodept_ast_nodes::{Literal, Param, ResolvedType, UserType, ValueCtor};
use kodept_core::code_point::Span;
use kodept_ecs::component::Component;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::system::{Commands, Query};
use kodept_ecs::world::Ref;
use kodept_inference::process::PartialInfer;
use kodept_inference::r#type::{MonomorphicType, PrimitiveType, TConstant};
use kodept_report::message::Diagnostic;
use kodept_report::prelude::Severity;
use kodept_report_macros::Report;
use num_bigint::Sign;
use std::num::NonZeroU8;

#[derive(Debug, Component)]
pub(super) struct PartiallyTypechecked(PartialInfer<(NodeId, SymbolKind)>);
#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
#[component(immutable)]
pub(super) struct Foo(MonomorphicType);

impl NodeProperty for PartiallyTypechecked {}
impl RequireProperty<PartiallyTypechecked> for Literal {}

impl NodeProperty for Foo {}
impl RequireProperty<Foo> for UserType {}

#[derive(Debug, Report)]
#[severity("error")]
#[message("Integer literal is too big to fit into 256 bits")]
struct IntegerIsTooBig {
    span: Span,
}

pub(super) fn typeck_literals(
    query: Query<(NodeId<Literal>, &Literal, &SourceSpan)>,
    mut commands: Commands,
    mut reporter: Reporter,
) {
    for (id, literal, span) in query {
        let ty = match literal {
            Literal::Integer(x) => {
                let Ok(bits) = u8::try_from(x.bits()) else {
                    reporter.report(IntegerIsTooBig { span: span.0 });
                    continue;
                };
                match (x.sign(), NonZeroU8::new(bits)) {
                    (Sign::NoSign, None) => PrimitiveType::custom(false, NonZeroU8::MIN),
                    (Sign::Minus, Some(bits)) => PrimitiveType::custom(true, bits),
                    (Sign::Plus, Some(bits)) => PrimitiveType::custom(false, bits),
                    (_, None) => unreachable!("Signed value has no bits"),
                    (Sign::NoSign, Some(_)) => unreachable!("Zero has some bits"),
                }
            }
            Literal::Floating(_) => {
                // TODO: check whether value will fit into F24
                PrimitiveType::f24()
            }
            Literal::Char(_) => PrimitiveType::u8(),
            Literal::String(x) => PrimitiveType::string(x.len() as u64),
        };

        commands
            .entity(id.entity())
            .insert(PartiallyTypechecked(PartialInfer::new(ty)));
    }
}

pub(super) fn typeck_user_types(query: Query<NodeId<UserType>>, mut commands: Commands) {
    for id in query {
        let constant = TConstant::new();
        commands
            .entity(id.entity())
            .insert(Foo(MonomorphicType::Constant(constant)));
    }
}

fn resolved_ty_as_monomorphic(
    ty: &ResolvedType,
    all_types: Query<&Foo>,
) -> Option<MonomorphicType> {
    match ty {
        ResolvedType::Named(id) => all_types.get(id.entity()).ok().map(|it| it.0.clone()),
        ResolvedType::Tuple(items) => {
            let collection = items
                .iter()
                .map(|it| resolved_ty_as_monomorphic(it, all_types))
                .collect::<Option<Vec<_>>>()?;
            Some(MonomorphicType::tuple(collection))
        }
    }
}

pub(super) fn typeck_value_ctors(
    query: NarrowHierarchicalQuery<
        UserType,
        ValueCtor<ResolvedType>,
        (),
        &Foo,
        (Ref<ValueCtor<ResolvedType>>, &SourceSpan),
    >,
    all_types: Query<&Foo>,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    fn single(
        output: &MonomorphicType,
        params: &[Param<ResolvedType>],
        all_types: Query<&Foo>,
    ) -> Option<MonomorphicType> {
        match params.split_first() {
            None => Some(output.clone()),
            Some((head, tail)) => {
                let head = resolved_ty_as_monomorphic(head.ty(), all_types)?;
                let tail = tail
                    .iter()
                    .map(|it| resolved_ty_as_monomorphic(it.ty(), all_types))
                    .collect::<Option<Vec<_>>>()?;

                Some(MonomorphicType::fun(head, tail, output.clone()))
            }
        }
    }

    for (_, Foo(ty), ctors) in query.iter_by_layers() {
        for (id, (ctor, span)) in ctors {
            match single(ty, &ctor.params, all_types) {
                None => reporter.report_ad_hoc(|| {
                    Diagnostic::new(Severity::Bug)
                        .with_message("Monomorphic type is unknown")
                        .with_primary_label("in constructor", span.0)
                }),
                Some(ty) => _ = commands.entity(id.entity()).insert(Foo(ty)),
            }
        }
    }
}
