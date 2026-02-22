use crate::per_file::symbols::SymbolKind;
use crate::source::collection::Reporter;
use kodept_ast::prelude::NodeId;
use kodept_ast::properties::{NodeProperty, RequireProperty, SourceSpan};
use kodept_ast_nodes::{Literal, Param, ResolvedType, UserType, ValueCtor};
use kodept_core::code_point::Span;
use kodept_ecs::component::Component;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::system::{Commands, Query};
use kodept_inference::assumption::TypeTable;
use kodept_inference::process::PartialInfer;
use kodept_inference::r#type::{MonomorphicType, PrimitiveType, TVar};
use kodept_report_macros::Report;
use num_bigint::Sign;
use std::num::NonZeroU8;

type Referral = (NodeId, SymbolKind);

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub(super) struct PartiallyTypechecked(PartialInfer<Referral>);

impl NodeProperty for PartiallyTypechecked {}
impl RequireProperty<PartiallyTypechecked> for Literal {}
impl RequireProperty<PartiallyTypechecked> for UserType {}
impl RequireProperty<PartiallyTypechecked> for ValueCtor<ResolvedType> {}

#[derive(Debug, Report)]
#[severity("error")]
#[message("Integer literal is too big to fit into 256 bits")]
struct IntegerIsTooBig {
    #[primary_label]
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

fn resolved_ty_as_monomorphic(ty: &ResolvedType) -> PartialInfer<Referral> {
    match ty {
        ResolvedType::Named(id) => {
            let var = TVar::new();
            PartialInfer::new(var).with_assumption((*id, SymbolKind::Type), var)
        }
        ResolvedType::Tuple(items) => {
            let mut result = PartialInfer::new(MonomorphicType::UNIT);
            let ty = items
                .iter()
                .map(|it| resolved_ty_as_monomorphic(it))
                .map(|it| {
                    result.assumptions.merge(it.assumptions);
                    result.constraints.extend(it.constraints);
                    it.current_type.0
                });

            let monomorphic_type = MonomorphicType::tuple(ty);
            result.with_type(monomorphic_type)
        }
    }
}

pub(super) fn typeck_value_ctors(
    query: Query<(NodeId<ValueCtor<ResolvedType>>, &ValueCtor<ResolvedType>)>,
    mut commands: Commands,
) {
    for (id, ctor) in query {
        let named_params_count = ctor
            .params
            .iter()
            .rev()
            .take_while(|it| matches!(it, Param::Named { .. }))
            .count();
        let total_params_count = ctor.params.len();
        let (positional, named) = ctor
            .params
            .split_at(total_params_count - named_params_count);
        // TODO: add support for named parameters
        assert_eq!(named.len(), 0);

        let mut result = PartialInfer::new(MonomorphicType::UNIT);
        let ctor_ty = positional
            .iter()
            .map(|it| resolved_ty_as_monomorphic(it.ty()))
            .rfold(MonomorphicType::var(), |acc, next| {
                result.assumptions.merge(next.assumptions);
                result.constraints.extend(next.constraints);
                MonomorphicType::fun1(&*next.current_type, acc)
            });

        commands
            .entity(id.entity())
            .insert(PartiallyTypechecked(result.with_type(ctor_ty)));
    }
}
