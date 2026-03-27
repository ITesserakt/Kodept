use crate::per_file::symbols::{ResolvedTo, SymbolKind};
use crate::per_file::utils::{
    IterableSystem, IterableSystemParam, Modification, ParIterableSystem, Params, StaticQuery,
};
use crate::utils::TryReport;
use kodept_ast::prelude::{
    HierarchicalQuery, MutProperty, NarrowHierarchicalQuery, NodeId, Property,
};
use kodept_ast::properties::{NodeProperty, RequireProperty, SourceSpan};
use kodept_ast::relationship::NodeRelationship;
use kodept_ast::syntax_tree::children::{Family, MembersOf, Wrapper};
use kodept_ast::syntax_tree::experimental::{Buffer, NodeModification};
use kodept_ast_nodes::{
    AnonFunction, Branch, Call, Condition, Else, Expression, If, Lhs, Link, Literal,
    NormalizedBlock, Otherwise, Param, ResolvedTypeAnnotation, Rhs, Statement, Tuple, UserFunction,
    Value, Variable,
};
use kodept_core::code_point::Span;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{Component, ComponentIdFor};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::hierarchy::ChildOf;
use kodept_ecs::query::{AnyOf, Has, Without};
use kodept_ecs::relationship::Relationship;
use kodept_ecs::system::{ParamSet, Query, SystemParam};
use kodept_inference::constraint::{eq_cst, implicit_cst};
use kodept_inference::process::PartialInfer;
use kodept_inference::r#type::{MonomorphicType, PrimitiveType, TVar};
use kodept_report_macros::IntoMessage;
use num_bigint::Sign;
use std::collections::HashSet;
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
impl RequireProperty<PartiallyTypechecked> for Variable {}

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
            if let None = self.partials.get(child_id.entity()).ok().flatten() {
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
        for (branch_id, ()) in self.branches.get_children(modification.id()) {
            let branch_conditions = self.param_set.p0();
            let (_, Some(_)) = branch_conditions.get_children(branch_id).collect() else {
                return ();
            };
            let branch_bodies = self.param_set.p1();
            let (_, Some(_)) = branch_bodies.get_children(branch_id).collect() else {
                return ();
            };
        }
        let mut partial = match self.otherwise.get_children(modification.id()).collect() {
            Some((otherwise_id, ())) => {
                let mut otherwise_bodies = self.param_set.p2();
                let (_, Some(mut body)) = otherwise_bodies.get_children_mut(otherwise_id).collect()
                else {
                    return ();
                };
                body.take()
            }
            None => PartialInfer::new(MonomorphicType::UNIT),
        };

        for (branch_id, ()) in self.branches.get_children(modification.id()) {
            let mut branch_conditions = self.param_set.p0();
            let (_, condition) = branch_conditions.get_children_mut(branch_id).collect();
            let condition = condition.unwrap().take();

            let mut branch_bodies = self.param_set.p1();
            let (_, body) = branch_bodies.get_children_mut(branch_id).collect();
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
            .get_children(modification.id())
            .into_iter()
            .any(|it| it.1.is_none())
        {
            return ();
        }
        let rhs = self.param_set.p1();
        if rhs
            .get_children(modification.id())
            .into_iter()
            .any(|it| it.1.is_none())
        {
            return ();
        }

        let tv = TVar::new();
        let mut partial = PartialInfer::new(tv);

        let lhs_ty = {
            let mut lhs_fetch = self.param_set.p0();
            let (_, lhs) = lhs_fetch.get_children_mut(modification.id()).collect();
            let lhs_partial = lhs.unwrap().take();

            partial.merge(lhs_partial).0
        };

        let mut rhs = self.param_set.p1();
        let mut inputs = vec![];
        for (_, rhs) in rhs.get_children_mut(modification.id()) {
            let rhs = rhs.unwrap().take();
            inputs.push(partial.merge(rhs).0);
        }
        let expected_func_ty = match &*inputs {
            [] => MonomorphicType::fun1(MonomorphicType::UNIT, tv),
            _ => inputs
                .into_iter()
                .rfold(MonomorphicType::from(tv), |acc, next| {
                    MonomorphicType::fun1(next, acc)
                }),
        };

        partial.constraints.push(eq_cst(lhs_ty, expected_func_ty));
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
        let (_, Some(mut expr)) = self.links.get_children_mut(modification.id()).collect() else {
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
            .get_children(modification.id())
            .into_iter()
            .any(|it| it.1.1.is_none())
        {
            return ();
        }
        let tv = TVar::new();
        let mut result = PartialInfer::new(tv);

        for (_, (archetype, partial)) in self.statements.get_children_mut(modification.id()) {
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
                _ = result.merge(partial);
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
        (Property<ResolvedTypeAnnotation>, Property<ParamTyStub>),
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
        let (_, Some(mut body_partial)) = self.bodies.get_children_mut(modification.id()).collect()
        else {
            return ();
        };

        let mut body_ty = body_partial.take();

        let mut inputs_iter = self.params.get_children(modification.id()).into_iter().map(
            |(param_id, (bound, stub))| {
                let tv = stub.0;
                let param_assumptions = body_ty
                    .assumptions
                    .resolve_take(&(param_id.cast(), SymbolKind::Parameter));
                for assumption in param_assumptions.into_iter() {
                    body_ty.constraints.push(eq_cst(assumption.0, tv));
                }
                apply_bound(bound, &mut body_ty, tv.into());
                tv
            },
        );

        let expected_func_ty = match inputs_iter.next() {
            None => MonomorphicType::fun1(MonomorphicType::UNIT, body_ty.current_type.0),
            Some(head) => {
                let inputs = inputs_iter.collect::<Vec<_>>();
                MonomorphicType::fun(head, inputs, body_ty.current_type.0.clone())
            }
        };

        modification.add_property(PartiallyTypechecked::from(
            PartialInfer::new(expected_func_ty)
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
        (Property<ResolvedTypeAnnotation>, Property<ParamTyStub>),
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
        let (_, Some(mut block_partial)) = self.block.get_children_mut(modification.id()).collect()
        else {
            return ();
        };
        let mut block_ty = block_partial.take();

        let output_ty = block_ty.current_type.0.clone();
        let mut inputs_iter = self.params.get_children(modification.id()).into_iter().map(
            |(param_id, (bound, stub))| {
                let tv = stub.0;
                let param_assumptions = block_ty
                    .assumptions
                    .resolve_take(&(param_id.cast(), SymbolKind::Parameter));
                for assumption in param_assumptions.into_iter() {
                    block_ty.constraints.push(eq_cst(assumption.0, tv));
                }
                apply_bound(bound, &mut block_ty, tv.into());
                tv
            },
        );

        let expected_func_ty = match inputs_iter.next() {
            None => MonomorphicType::fun1(MonomorphicType::UNIT, block_ty.current_type.0),
            Some(head) => {
                let inputs = inputs_iter.collect::<Vec<_>>();
                MonomorphicType::fun(head, inputs, output_ty.clone())
            }
        };

        apply_bound(params.0, &mut block_ty, output_ty);

        modification.add_property(PartiallyTypechecked::from(
            PartialInfer::new(expected_func_ty)
                .with_assumptions(block_ty.assumptions)
                .with_constraints(block_ty.constraints),
        ));
    }
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
#[component(immutable)]
struct ParamTyStub(TVar);

impl NodeProperty for ParamTyStub {}
impl RequireProperty<ParamTyStub> for Param {}

#[derive(SystemParam)]
pub(super) struct FillParamTyStubs;

impl ParIterableSystem for FillParamTyStubs {
    type Iterable = StaticQuery<(NodeId<Param>,)>;

    fn for_each<B: Buffer>(
        &self,
        mut modification: Modification<Self::Iterable, B>,
        _: Params<Self::Iterable>,
    ) -> impl TryReport {
        let tv = TVar::new();
        modification.add_property(ParamTyStub(tv));
    }
}

type ChildrenFor<P, Tag> = <<P as NodeRelationship<Tag, <P as Family<Tag>>::Arity>>::Relationship as Relationship>::RelationshipTarget;

#[derive(SystemParam)]
pub(super) struct TypeckVariable<'w, 's> {
    ps: ParamSet<
        'w,
        's,
        (
            (
                Query<
                    'w,
                    's,
                    (
                        NodeId,
                        &'static ChildOf,
                        Has<UserFunction>,
                        Has<AnonFunction>,
                    ),
                >,
                StaticQuery<&'static ParamTyStub>,
                StaticQuery<
                    Option<
                        AnyOf<(
                            &'static ChildrenFor<UserFunction, kodept_ast_nodes::Params>,
                            &'static ChildrenFor<AnonFunction, kodept_ast_nodes::Params>,
                        )>,
                    >,
                >,
            ),
            StaticQuery<&'static mut PartiallyTypechecked>,
        ),
    >,
}

impl IterableSystem for TypeckVariable<'_, '_> {
    type Iterable = HierarchicalQuery<
        'static,
        'static,
        Variable,
        Expression,
        (),
        (),
        Without<PartiallyTypechecked>,
    >;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
    ) -> impl TryReport {
        let ((), expr_fetch) = params;
        let (expr_id, ()) = expr_fetch.collect();
        let mut expr_partial = {
            let mut p1 = self.ps.p1();
            let Ok(mut expr_partial) = p1.get_mut(expr_id.entity()) else {
                return ();
            };
            expr_partial.take()
        };
        let (parents, param_stubs, params) = self.ps.p0();

        let mut current = modification.id().entity();
        let mut monomorphic_ctx = HashSet::new();
        loop {
            let Ok((id, &ChildOf(parent), is_user_function, is_anon_function)) =
                parents.get(current)
            else {
                break;
            };
            current = parent;
            if is_user_function || is_anon_function {
                let children = match params.get(id.entity()).unwrap() {
                    None => continue,
                    Some((Some(children), Some(_))) => children,
                    Some((Some(children), None)) | Some((None, Some(children))) => children,
                    _ => unreachable!(),
                };
                let param_stubs_fetch = param_stubs.iter_many(children);
                monomorphic_ctx.extend(param_stubs_fetch.map(|it| it.0));
            }
        }

        let tys = expr_partial
            .assumptions
            .resolve_take(&(modification.id().cast(), SymbolKind::Variable));
        let im_cs = tys
            .into_iter()
            .map(|it| implicit_cst(it.0, monomorphic_ctx.clone(), expr_partial.current_type.0));

        let resulting_partial = PartialInfer::new(MonomorphicType::UNIT)
            .with_constraints(im_cs)
            .with_constraints(expr_partial.constraints)
            .with_assumptions(expr_partial.assumptions);
        modification.add_property(PartiallyTypechecked::from(resulting_partial));
    }
}
