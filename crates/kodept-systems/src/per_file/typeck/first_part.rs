use crate::per_file::symbols::{ResolvedTo, SymbolKind};
use crate::per_file::utils::{
    IterableSystem, IterableSystemParam, Modification, ParIterableSystem, Params, StaticQuery,
};
use crate::utils::TryReport;
use kodept_ast::prelude::{
    HierarchicalQuery, MutProperty, NarrowHierarchicalQuery, NodeId, Property,
};
use kodept_ast::properties::{NodeProperty, RequireProperty, SourceSpan};
use kodept_ast::syntax_tree::children::{MembersOf, Wrapper};
use kodept_ast::syntax_tree::experimental::{Buffer, NodeModification};
use kodept_ast_nodes::{
    AnonFunction, Branch, Call, Condition, Else, Expression, If, Lhs, Link, Literal,
    NormalizedBlock, Otherwise, Param, ResolvedTypeAnnotation, Rhs, Statement, Tuple, UserFunction,
    Value,
};
use kodept_core::code_point::Span;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{Component, ComponentIdFor};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::Without;
use kodept_ecs::system::{ParamSet, Query, SystemParam};
use kodept_inference::constraint::eq_cst;
use kodept_inference::process::PartialInfer;
use kodept_inference::r#type::{MonomorphicType, PrimitiveType, TVar};
use kodept_report_macros::IntoMessage;
use num_bigint::Sign;
use std::marker::PhantomData;
use std::num::NonZeroU8;

type Referral = (NodeId, SymbolKind);

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub(super) struct PartiallyTypechecked(Option<PartialInfer<Referral>>);

impl PartiallyTypechecked {
    fn new(ty: MonomorphicType) -> Self {
        Self(Some(PartialInfer::new(ty)))
    }

    pub(super) fn take(&mut self) -> PartialInfer<Referral> {
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
impl RequireProperty<PartiallyTypechecked> for Call {}
impl RequireProperty<PartiallyTypechecked> for Link {}
impl RequireProperty<PartiallyTypechecked> for AnonFunction {}
impl RequireProperty<PartiallyTypechecked> for UserFunction {}

#[derive(Debug, IntoMessage)]
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
            .map(|it| partial.merge(it).0);
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

            let condition_ty = partial.merge(condition);
            let body_ty = partial.merge(body);
            partial.constraints.extend([
                eq_cst(condition_ty.0, PrimitiveType::Boolean),
                eq_cst(body_ty.0, partial.current_type.0),
            ]);
        }

        modification.add_property(PartiallyTypechecked::from(partial));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckCall<'w, 's> {
    param_set: ParamSet<
        'w,
        's,
        (
            HierarchicalQuery<
                'static,
                'static,
                Call,
                Lhs,
                (),
                Option<MutProperty<PartiallyTypechecked>>,
            >,
            HierarchicalQuery<
                'static,
                'static,
                Call,
                Rhs,
                (),
                Option<MutProperty<PartiallyTypechecked>>,
            >,
        ),
    >,
}

impl IterableSystem for TypeckCall<'_, '_> {
    type Iterable = StaticQuery<(NodeId<Call>,), Without<PartiallyTypechecked>>;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        _: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let lhs = self.param_set.p0();
        if lhs
            .get_down(modification.id())
            .1
            .into_iter()
            .any(|it| it.1.is_none())
        {
            return ();
        }
        let rhs = self.param_set.p1();
        if rhs
            .get_down(modification.id())
            .1
            .into_iter()
            .any(|it| it.1.is_none())
        {
            return ();
        }

        let tv = TVar::new();
        let mut partial = PartialInfer::new(tv);

        let lhs_ty = {
            let mut lhs_fetch = self.param_set.p0();
            let ((), lhs_fetch) = lhs_fetch.get_down_mut(modification.id());
            let (_, lhs) = lhs_fetch.collect();
            let lhs_partial = lhs.unwrap().take();

            partial.merge(lhs_partial).0
        };

        let mut rhs = self.param_set.p1();
        let mut inputs = vec![];
        for (_, rhs) in rhs.get_down_mut(modification.id()).1 {
            let rhs = rhs.unwrap().take();
            inputs.push(partial.merge(rhs).0);
        }
        if inputs.is_empty() {
            partial.constraints.push(eq_cst(
                lhs_ty,
                MonomorphicType::fun1(MonomorphicType::UNIT, tv),
            ));
        } else {
            let func_ty = inputs
                .into_iter()
                .rfold(MonomorphicType::from(tv), |acc, next| {
                    MonomorphicType::fun1(next, acc)
                });
            partial.constraints.push(eq_cst(lhs_ty, func_ty));
        }

        modification.add_property(PartiallyTypechecked::from(partial));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckLink<'w, 's> {
    links: HierarchicalQuery<
        'w,
        's,
        Link,
        Expression,
        (),
        Option<MutProperty<PartiallyTypechecked>>,
        Without<PartiallyTypechecked>,
    >,
}

impl IterableSystem for TypeckLink<'_, '_> {
    type Iterable = StaticQuery<(NodeId<Link>,), Without<PartiallyTypechecked>>;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        _: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let ((), expr_fetch) = self.links.get_down_mut(modification.id());
        let (_, Some(mut expr)) = expr_fetch.collect() else {
            return ();
        };
        let partial = expr.take();
        modification.add_property(PartiallyTypechecked::from(partial));
    }
}

struct ComponentIdWrapper<'a>(PhantomData<&'a ()>);
impl<'a> Wrapper for ComponentIdWrapper<'a> {
    type Wrapped<T: Component> = ComponentIdFor<'a, T>;
}

#[derive(SystemParam)]
pub(super) struct TypeckBlock<'w, 's> {
    statements: HierarchicalQuery<
        'w,
        's,
        NormalizedBlock,
        Statement,
        (),
        (
            &'static Archetype,
            Option<MutProperty<PartiallyTypechecked>>,
        ),
        Without<PartiallyTypechecked>,
    >,
    statement_component_ids: MembersOf<NormalizedBlock, Statement, ComponentIdWrapper<'s>>,
}

impl IterableSystem for TypeckBlock<'_, '_> {
    type Iterable = StaticQuery<(NodeId<NormalizedBlock>,), Without<PartiallyTypechecked>>;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        _: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        if self
            .statements
            .get_down(modification.id())
            .1
            .into_iter()
            .any(|it| it.1.1.is_none())
        {
            return ();
        }
        let tv = TVar::new();
        let mut result = PartialInfer::new(tv);

        for (_, (archetype, partial)) in self.statements.get_down_mut(modification.id()).1 {
            let partial = partial.unwrap().take();
            if archetype.contains(self.statement_component_ids.0.get()) {
                // statement is block
                result.merge(partial);
            } else if archetype.contains(self.statement_component_ids.1.get()) {
                // statement is call
                result.merge(partial);
            } else if archetype.contains(self.statement_component_ids.2.get()) {
                // statement is if
                result.merge(partial);
            } else if archetype.contains(self.statement_component_ids.3.get()) {
                // statement is link
                let link_ty = result.merge(partial);
                result.constraints.push(eq_cst(tv, link_ty.0));
            } else if archetype.contains(self.statement_component_ids.4.get()) {
                // statement is user function
            } else if archetype.contains(self.statement_component_ids.5.get()) {
                // statement is variable
            }
        }

        modification.add_property(PartiallyTypechecked::from(result));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckAnonFunction<'w, 's> {
    bodies: NarrowHierarchicalQuery<
        'w,
        's,
        AnonFunction,
        NormalizedBlock,
        (),
        (),
        Option<MutProperty<PartiallyTypechecked>>,
        Without<PartiallyTypechecked>,
    >,
    params: NarrowHierarchicalQuery<
        'w,
        's,
        AnonFunction,
        Param,
        kodept_ast_nodes::Params,
        (),
        Property<ResolvedTypeAnnotation>,
    >,
}

fn apply_bound(
    bound: &ResolvedTypeAnnotation,
    partial: &mut PartialInfer<Referral>,
    tv: MonomorphicType,
) {
    match bound {
        ResolvedTypeAnnotation::Infer => {}
        &ResolvedTypeAnnotation::Named(x) => {
            partial.assumptions.push_single((x, SymbolKind::Type), tv)
        }
        ResolvedTypeAnnotation::Tuple(items) => {
            let tuple = items.iter().map(|it| {
                let tp = TVar::new();
                apply_bound(it, partial, tp.into());
                tp
            });
            let tu = MonomorphicType::tuple(tuple);
            partial.constraints.push(eq_cst(tv, tu));
        }
    }
}

impl IterableSystem for TypeckAnonFunction<'_, '_> {
    type Iterable = StaticQuery<(NodeId<AnonFunction>,), Without<PartiallyTypechecked>>;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: Modification<Self::Iterable, B>,
        _: Params<Self::Iterable>,
    ) -> impl TryReport {
        let ((), body_fetch) = self.bodies.get_down_mut(modification.id());
        let (_, Some(mut body_partial)) = body_fetch.collect() else {
            return ();
        };

        let inputs = self
            .params
            .get_down(modification.id())
            .1
            .into_iter()
            .map(|_| TVar::new())
            .collect::<Vec<_>>();
        let mut body_ty = body_partial.take();

        for (&tv, (param_id, bound)) in inputs.iter().zip(self.params.get_down(modification.id()).1)
        {
            let param_assumptions = body_ty
                .assumptions
                .resolve_take((param_id.cast(), SymbolKind::Parameter));
            for assumption in param_assumptions.into_iter() {
                body_ty.constraints.push(eq_cst(assumption.0, tv));
            }
            apply_bound(bound, &mut body_ty, tv.into());
        }

        let func_ty = if inputs.is_empty() {
            MonomorphicType::fun1(MonomorphicType::UNIT, body_ty.current_type.0)
        } else {
            inputs
                .into_iter()
                .rfold(body_ty.current_type.0.clone(), |acc, next| {
                    MonomorphicType::fun1(next, acc)
                })
        };

        modification.add_property(PartiallyTypechecked::from(
            PartialInfer::new(func_ty)
                .with_assumptions(body_ty.assumptions)
                .with_constraints(body_ty.constraints),
        ));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckUserFunction<'w, 's> {
    block: NarrowHierarchicalQuery<
        'w,
        's,
        UserFunction,
        NormalizedBlock,
        (),
        (),
        Option<MutProperty<PartiallyTypechecked>>,
        Without<PartiallyTypechecked>,
    >,
    params: NarrowHierarchicalQuery<
        'w,
        's,
        UserFunction,
        Param,
        kodept_ast_nodes::Params,
        (),
        Property<ResolvedTypeAnnotation>,
        Without<PartiallyTypechecked>,
    >,
}

impl IterableSystem for TypeckUserFunction<'_, '_> {
    type Iterable = StaticQuery<
        (NodeId<UserFunction>, Property<ResolvedTypeAnnotation>),
        Without<PartiallyTypechecked>,
    >;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
    ) -> impl TryReport {
        let ((), block_fetch) = self.block.get_down_mut(modification.id());
        let (_, Some(mut block_partial)) = block_fetch.collect() else {
            return ();
        };
        let mut block_ty = block_partial.take();

        let inputs = self
            .params
            .get_down(modification.id())
            .1
            .into_iter()
            .map(|_| TVar::new())
            .collect::<Vec<_>>();

        for (&tv, (param_id, bound)) in inputs.iter().zip(self.params.get_down(modification.id()).1)
        {
            let param_assumptions = block_ty
                .assumptions
                .resolve_take((param_id.cast(), SymbolKind::Parameter));
            for assumption in param_assumptions.into_iter() {
                block_ty.constraints.push(eq_cst(assumption.0, tv));
            }
            apply_bound(bound, &mut block_ty, tv.into());
        }

        let output_ty = block_ty.current_type.0.clone();
        let func_ty = if inputs.is_empty() {
            MonomorphicType::fun1(MonomorphicType::UNIT, output_ty.clone())
        } else {
            inputs.into_iter().rfold(output_ty.clone(), |acc, next| {
                MonomorphicType::fun1(next, acc)
            })
        };

        apply_bound(params.0, &mut block_ty, output_ty);

        modification.add_property(PartiallyTypechecked::from(
            PartialInfer::new(func_ty)
                .with_assumptions(block_ty.assumptions)
                .with_constraints(block_ty.constraints),
        ));
    }
}
