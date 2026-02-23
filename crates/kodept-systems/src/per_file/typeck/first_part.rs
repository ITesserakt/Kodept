use crate::per_file::symbols::{ResolvedTo, SymbolKind};
use crate::per_file::utils::{IterableSystem, IterableSystemParam, ParIterableSystem, StaticQuery};
use crate::utils::TryReport;
use kodept_ast::prelude::{
    HierarchicalQuery, MutProperty, NarrowHierarchicalQuery, NodeId, Property,
};
use kodept_ast::properties::{NodeProperty, RequireProperty, SourceSpan};
use kodept_ast::syntax_tree::experimental::{Buffer, NodeModification};
use kodept_ast_nodes::{
    Branch, Condition, Else, Expression, If, Literal, NormalizedBlock, Otherwise, ResolvedType,
    Tuple, Value,
};
use kodept_core::code_point::Span;
use kodept_ecs::component::Component;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::Without;
use kodept_ecs::system::{ParamSet, Query, SystemParam};
use kodept_inference::assumption::TypeTable;
use kodept_inference::constraint::eq_cst;
use kodept_inference::process::PartialInfer;
use kodept_inference::r#type::{MonomorphicType, PrimitiveType, TVar};
use kodept_report_macros::Report;
use num_bigint::Sign;
use std::num::NonZeroU8;

type Referral = (NodeId, SymbolKind);

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub(super) struct PartiallyTypechecked(Option<PartialInfer<Referral>>);

impl PartiallyTypechecked {
    fn new(ty: MonomorphicType) -> Self {
        Self(Some(PartialInfer::new(ty)))
    }

    fn take(&mut self) -> PartialInfer<Referral> {
        self.0.take().expect("Cannot take partial")
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

impl From<PartialInfer<Referral>> for PartiallyTypechecked {
    fn from(value: PartialInfer<Referral>) -> Self {
        Self(Some(value))
    }
}

impl NodeProperty for PartiallyTypechecked {}
impl RequireProperty<PartiallyTypechecked> for Literal {}
impl RequireProperty<PartiallyTypechecked> for Value {}
impl RequireProperty<PartiallyTypechecked> for Tuple {}
impl RequireProperty<PartiallyTypechecked> for If {}
impl RequireProperty<PartiallyTypechecked> for NormalizedBlock {}

#[derive(Debug, Report)]
#[severity("error")]
#[message("Integer literal is too big to fit into 256 bits")]
struct IntegerIsTooBig {
    #[primary_label]
    span: Span,
}

#[derive(SystemParam)]
pub(super) struct TypeckLiteral;

impl ParIterableSystem for TypeckLiteral {
    type Iterable = StaticQuery<(NodeId<Literal>, &'static Literal, Property<SourceSpan>)>;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let (literal, &SourceSpan(span)) = params;
        let ty = match literal {
            Literal::Integer(x) => {
                let Ok(bits) = u8::try_from(x.bits()) else {
                    return Err(IntegerIsTooBig { span });
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

        modification.add_property(PartiallyTypechecked::new(ty.into()));

        Ok(())
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckValue;

impl ParIterableSystem for TypeckValue {
    type Iterable = StaticQuery<(NodeId<Value>, Property<ResolvedTo>)>;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let (&ResolvedTo { referral, kind },) = params;

        let var = TVar::new();
        let partial = PartialInfer::new(var).with_assumption((referral, kind), var);
        modification.add_property(PartiallyTypechecked::from(partial));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckTuple<'w, 's> {
    partials: Query<'w, 's, Option<&'static mut PartiallyTypechecked>>,
}

impl IterableSystem for TypeckTuple<'_, '_> {
    type Iterable = HierarchicalQuery<
        'static,
        'static,
        Tuple,
        Expression,
        (),
        (),
        Without<PartiallyTypechecked>,
    >;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let ((), children) = params;
        for (child_id, ()) in &children {
            if let None = self.partials.get_mut(child_id.entity()).ok().flatten() {
                return ();
            }
        }
        let mut partial = PartialInfer::new(MonomorphicType::UNIT);
        let tuple_ty = children
            .into_iter()
            .filter_map(|(child_id, ())| {
                let mut partial = self.partials.get_mut(child_id.entity()).ok()??;
                Some(partial.take())
            })
            .map(|it| {
                partial.assumptions.merge(it.assumptions);
                partial.constraints.extend(it.constraints);
                it.current_type.0
            });
        let tuple_ty = MonomorphicType::tuple(tuple_ty);
        let partial = partial.with_type(tuple_ty);

        modification.add_property(PartiallyTypechecked::from(partial));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckIf<'w, 's> {
    branches:
        NarrowHierarchicalQuery<'w, 's, If, Branch, (), (), (), Without<PartiallyTypechecked>>,
    otherwise:
        NarrowHierarchicalQuery<'w, 's, If, Otherwise, Else, (), (), Without<PartiallyTypechecked>>,
    param_set: ParamSet<
        'w,
        's,
        (
            HierarchicalQuery<
                'static,
                'static,
                Branch,
                Condition,
                (),
                Option<MutProperty<PartiallyTypechecked>>,
            >,
            HierarchicalQuery<
                'static,
                'static,
                Branch,
                Expression,
                (),
                Option<MutProperty<PartiallyTypechecked>>,
            >,
            HierarchicalQuery<
                'static,
                'static,
                Otherwise,
                Expression,
                (),
                Option<MutProperty<PartiallyTypechecked>>,
            >,
        ),
    >,
}

impl IterableSystem for TypeckIf<'_, '_> {
    type Iterable = StaticQuery<(NodeId<If>,), Without<PartiallyTypechecked>>;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        _: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        for (branch_id, ()) in self.branches.get_down(modification.id()).1 {
            let branch_conditions = self.param_set.p0();
            let ((), condition) = branch_conditions.get_down(branch_id);
            let (_, Some(_)) = condition.collect() else {
                return ();
            };
            let branch_bodies = self.param_set.p1();
            let ((), body) = branch_bodies.get_down(branch_id);
            let (_, Some(_)) = body.collect() else {
                return ();
            };
        }
        let ((), otherwise_fetch) = self.otherwise.get_down(modification.id());
        let mut partial = if let Some((otherwise_id, ())) = otherwise_fetch.collect() {
            let mut otherwise_bodies = self.param_set.p2();
            let ((), body) = otherwise_bodies.get_down_mut(otherwise_id);
            let (_, Some(mut body)) = body.collect() else {
                return ();
            };
            body.take()
        } else {
            PartialInfer::new(MonomorphicType::UNIT)
        };

        for (branch_id, ()) in self.branches.get_down(modification.id()).1 {
            let mut branch_conditions = self.param_set.p0();
            let ((), condition) = branch_conditions.get_down_mut(branch_id);
            let (_, condition) = condition.collect();
            let condition = condition.unwrap().take();

            let mut branch_bodies = self.param_set.p1();
            let ((), body) = branch_bodies.get_down_mut(branch_id);
            let (_, body) = body.collect();
            let body = body.unwrap().take();

            partial.assumptions.merge(condition.assumptions);
            partial.assumptions.merge(body.assumptions);
            partial.constraints.extend(condition.constraints);
            partial.constraints.extend(body.constraints);
            partial.constraints.extend([
                eq_cst(condition.current_type.0, PrimitiveType::Boolean),
                eq_cst(body.current_type.0, partial.current_type.0),
            ]);
        }

        modification.add_property(PartiallyTypechecked::from(partial));
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
