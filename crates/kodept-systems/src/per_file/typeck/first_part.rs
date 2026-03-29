use crate::per_file::symbols::{ResolvedTo, SymbolKind};
use crate::per_file::utils::{
    IterableSystem, IterableSystemParam, Modification, ParIterableSystem, Params, StaticQuery,
};
use crate::utils::TryReport;
use kodept_ast::arity::{Plural, Singular};
use kodept_ast::prelude::{
    HierarchicalQuery, MutProperty, NarrowHierarchicalQuery, NodeId, Property,
};
use kodept_ast::properties::{HasProperty, NodeProperty, RequireProperty, SourceSpan};
use kodept_ast::syntax_tree::children::{HasChild, MembersOf, Wrapper};
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
use kodept_ecs::query::{Has, Without};
use kodept_ecs::system::{ParamSet, Query, SystemParam};
use kodept_inference::constraint::{eq_cst, implicit_cst};
use kodept_inference::engine::{DefaultTypeckContext, LangItem, TypeckContext};
use kodept_inference::process::PartialInfer;
use kodept_inference::r#type::{MonomorphicType, PolymorphicType, PrimitiveType, TVar};
use kodept_report_macros::IntoMessage;
use num_bigint::Sign;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::num::NonZeroU8;
use std::sync::Arc;

type Referral = (NodeId, SymbolKind);

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub(super) struct PartiallyTypechecked(Option<PartialInfer<Referral>>);

impl PartiallyTypechecked {
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
    type Iterable = StaticQuery<
        (NodeId<Literal>, &'static Literal, Property<SourceSpan>),
        Without<PartiallyTypechecked>,
    >;

    fn for_each<B: Buffer>(
        &self,
        mut modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
    ) -> impl TryReport {
        let (literal, &SourceSpan(span)) = params;

        let literal_type = match literal {
            Literal::Integer(x) => match u8::try_from(x.bits()) {
                Ok(bits) => match (x.sign(), NonZeroU8::new(bits)) {
                    (Sign::NoSign, None) => PrimitiveType::custom(false, NonZeroU8::MIN),
                    (_, None) => panic!("Signed value has no bits: {x}"),
                    (Sign::NoSign, _) => panic!("Zero value with bits: {x}"),
                    (Sign::Plus, Some(bits)) => PrimitiveType::custom(false, bits),
                    (Sign::Minus, Some(bits)) => PrimitiveType::custom(true, bits),
                },
                Err(_) => return Err(IntegerIsTooBig { span }),
            },
            Literal::Floating(_) => PrimitiveType::f24(),
            Literal::Char(_) => PrimitiveType::u8(),
            Literal::String(x) => PrimitiveType::string(x.len() as u64),
        };

        let item = LangItem::Type(PolymorphicType::from(literal_type));
        let partial = DefaultTypeckContext.eval(item);
        modification.add_property(PartiallyTypechecked::from(partial));
        Ok(())
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckValue;

impl ParIterableSystem for TypeckValue {
    type Iterable = StaticQuery<(NodeId<Value>, Property<ResolvedTo>)>;

    fn for_each<B: Buffer>(
        &self,
        mut modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
    ) -> impl TryReport {
        let (ResolvedTo { referral, kind },) = params;
        let item = LangItem::Value((*referral, *kind));
        let partial = DefaultTypeckContext.eval(item);
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
        mut modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
    ) -> impl TryReport {
        let ((), children) = params;
        if children
            .iter()
            .map(|it| self.partials.get(it.0.entity()).ok().flatten())
            .any(|it| it.is_none())
        {
            return ();
        }

        let item = LangItem::Tuple(Box::new(children.into_iter().map(|it| {
            let value = self.partials.get_mut(it.0.entity());
            value.unwrap().unwrap().take()
        })));
        let partial = DefaultTypeckContext.eval(item);
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
            let p0 = self.param_set.p0();
            let (_, Some(_)) = p0.get_children(branch_id).collect() else {
                return ();
            };
            let p1 = self.param_set.p1();
            let (_, Some(_)) = p1.get_children(branch_id).collect() else {
                return ();
            };
        }
        let otherwise = match self.otherwise.get_children(modification.id()).collect() {
            None => kodept_inference::engine::Branch {
                condition: PartialInfer::new(PrimitiveType::bool()),
                body: PartialInfer::new(MonomorphicType::UNIT),
            },
            Some((otherwise_id, ())) => {
                let mut p2 = self.param_set.p2();
                let (_, Some(mut body)) = p2.get_children_mut(otherwise_id).collect() else {
                    return ();
                };
                kodept_inference::engine::Branch {
                    condition: PartialInfer::new(PrimitiveType::bool()),
                    body: body.take(),
                }
            }
        };

        let item = LangItem::Branches(Box::new(
            self.branches
                .get_children(modification.id())
                .into_iter()
                .map(|(id, ())| {
                    let mut p0 = self.param_set.p0();
                    let (_, Some(mut condition)) = p0.get_children_mut(id).collect() else {
                        unreachable!()
                    };
                    let condition = condition.take();

                    let mut p1 = self.param_set.p1();
                    let (_, Some(mut body)) = p1.get_children_mut(id).collect() else {
                        unreachable!()
                    };
                    let body = body.take();

                    kodept_inference::engine::Branch { condition, body }
                })
                .chain([otherwise]),
        ));
        let partial = DefaultTypeckContext.eval(item);
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
        let p1 = self.param_set.p1();
        if p1
            .get_children(modification.id())
            .into_iter()
            .any(|it| it.1.is_none())
        {
            return ();
        }
        let callable = {
            let mut p0 = self.param_set.p0();
            let (_, Some(mut callable)) = p0.get_children_mut(modification.id()).collect() else {
                return ();
            };
            callable.take()
        };

        let mut p1 = self.param_set.p1();
        let item = LangItem::Call {
            callable,
            arguments: Box::new(
                p1.get_children_mut(modification.id())
                    .into_iter()
                    .map(|it| it.1.unwrap().take())
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
        };
        let partial = DefaultTypeckContext.eval(item);
        modification.add_property(PartiallyTypechecked::from(partial));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckLink;

impl ParIterableSystem for TypeckLink {
    type Iterable = HierarchicalQuery<
        'static,
        'static,
        Link,
        Expression,
        (),
        Option<MutProperty<PartiallyTypechecked>>,
        Without<PartiallyTypechecked>,
    >;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let ((), params_fetch) = params;
        let (_, Some(mut expr_partial)) = params_fetch.collect() else {
            return ();
        };
        let partial = expr_partial.take();
        let item = LangItem::Forward(partial);
        let partial = DefaultTypeckContext.eval(item);
        modification.add_property(PartiallyTypechecked::from(partial));
    }
}

struct ComponentIdWrapper<'a>(PhantomData<&'a ()>);
impl<'a> Wrapper for ComponentIdWrapper<'a> {
    type Wrapped<T: Component> = ComponentIdFor<'a, T>;
}

#[derive(SystemParam)]
pub(super) struct CollectMonomorphicContext<'w, 's> {
    parents: Query<
        'w,
        's,
        (
            NodeId,
            Option<&'static ChildOf>,
            Has<UserFunction>,
            Has<AnonFunction>,
        ),
    >,
    user_function_params: NarrowHierarchicalQuery<
        'w,
        's,
        UserFunction,
        Param,
        kodept_ast_nodes::Params,
        (),
        Property<ParamTyStub>,
    >,
    anon_function_params: NarrowHierarchicalQuery<
        'w,
        's,
        AnonFunction,
        Param,
        kodept_ast_nodes::Params,
        (),
        Property<ParamTyStub>,
    >,
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
#[component(immutable)]
pub(super) struct MonomorphicContext(Arc<HashSet<TVar>>);

impl NodeProperty for MonomorphicContext {}
impl RequireProperty<MonomorphicContext> for NormalizedBlock {}

impl ParIterableSystem for CollectMonomorphicContext<'_, '_> {
    type Iterable = StaticQuery<(NodeId<NormalizedBlock>,)>;

    fn for_each<B: Buffer>(
        &self,
        mut modification: Modification<Self::Iterable, B>,
        _: Params<Self::Iterable>,
    ) -> impl TryReport {
        let mut current = modification.id().entity();
        let mut ctx = HashSet::new();

        loop {
            let Ok((id, parent, is_user_fn, is_anon_fn)) = self.parents.get(current) else {
                return ();
            };
            let Some(&ChildOf(parent)) = parent else {
                break;
            };

            if is_anon_fn {
                let id = id.cast();
                let params = self.anon_function_params.get_children(id);
                ctx.extend(params.into_iter().map(|it| it.1.0));
            } else if is_user_fn {
                let id = id.cast();
                let params = self.user_function_params.get_children(id);
                ctx.extend(params.into_iter().map(|it| it.1.0));
                break;
            }

            current = parent;
        }

        modification.add_property(MonomorphicContext(Arc::new(ctx)));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckBlock<'s> {
    statement_component_ids: MembersOf<NormalizedBlock, Statement, ComponentIdWrapper<'s>>,
}

impl TypeckBlock<'_> {
    fn resolve_variables(
        statement_partial: &mut PartialInfer<Referral>,
        variables: &[(Referral, &MonomorphicType)],
        ctx: Arc<HashSet<TVar>>,
    ) {
        for (var_referral, var_type) in variables {
            let assumptions = statement_partial.assumptions.resolve_take(var_referral);
            statement_partial.constraints.extend(
                assumptions
                    .into_iter()
                    .map(|it| implicit_cst(it.0, &ctx, *var_type)),
            );
        }
    }
}

impl ParIterableSystem for TypeckBlock<'_> {
    type Iterable = HierarchicalQuery<
        'static,
        'static,
        NormalizedBlock,
        Statement,
        Property<MonomorphicContext>,
        (
            &'static Archetype,
            Option<MutProperty<PartiallyTypechecked>>,
        ),
        Without<PartiallyTypechecked>,
    >;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let (MonomorphicContext(ctx), statements) = params;
        if statements.iter().any(|it| it.1.1.is_none()) {
            return ();
        }
        let mut container = PartialInfer::new(TVar::new());
        let mut any_returns = false;
        let mut declared_variables = Vec::new();

        for (statement_id, (archetype, partial)) in statements {
            let mut partial = partial.unwrap().take();
            Self::resolve_variables(&mut partial, &declared_variables, ctx.clone());
            let statement_ty = container.merge(partial);

            if archetype.contains(self.statement_component_ids.3.get()) {
                // statement is a link
                any_returns = true;
                container
                    .constraints
                    .push(eq_cst(statement_ty.0, container.current_type.0));
            } else if archetype.contains(self.statement_component_ids.5.get()) {
                // statement is a variable
                declared_variables.push(((statement_id, SymbolKind::Variable), statement_ty.0));
            }
        }
        if !any_returns {
            container
                .constraints
                .push(eq_cst(MonomorphicType::UNIT, container.current_type.0));
        }
        modification.add_property(PartiallyTypechecked::from(container));
    }
}

#[derive(SystemParam)]
pub(super) struct TypeckFunction<'w, 's, T>
where
    T: HasChild<NormalizedBlock> + HasChild<Param, kodept_ast_nodes::Params>,
{
    bodies: NarrowHierarchicalQuery<
        'w,
        's,
        T,
        NormalizedBlock,
        (),
        (),
        Option<MutProperty<PartiallyTypechecked>>,
        Without<PartiallyTypechecked>,
    >,
    params: NarrowHierarchicalQuery<
        'w,
        's,
        T,
        Param,
        kodept_ast_nodes::Params,
        (),
        (Property<ResolvedTypeAnnotation>, Property<ParamTyStub>),
    >,
}

fn convert_annotation(
    input: &ResolvedTypeAnnotation,
) -> kodept_inference::engine::Annotation<'_, Referral> {
    match input {
        ResolvedTypeAnnotation::Infer => kodept_inference::engine::Annotation::None,
        ResolvedTypeAnnotation::Named(x) => {
            kodept_inference::engine::Annotation::Single((*x, SymbolKind::Type))
        }
        ResolvedTypeAnnotation::Tuple(vec) => kodept_inference::engine::Annotation::Tuple(
            Box::new(vec.iter().map(|x| convert_annotation(x))),
        ),
    }
}

impl<T> IterableSystem for TypeckFunction<'_, '_, T>
where
    T: HasChild<NormalizedBlock, Arity = Singular>,
    T: HasChild<Param, kodept_ast_nodes::Params, Arity = Plural>,
    T: HasProperty<PartiallyTypechecked>,
{
    type Iterable = StaticQuery<(NodeId<T>,), Without<PartiallyTypechecked>>;

    fn for_each<B: Buffer>(
        &mut self,
        mut modification: Modification<Self::Iterable, B>,
        _: Params<Self::Iterable>,
    ) -> impl TryReport {
        let body = {
            let (_, Some(mut body)) = self.bodies.get_children_mut(modification.id()).collect()
            else {
                return ();
            };
            body.take()
        };

        let fetch = self.params.get_children(modification.id());
        let item = LangItem::Lambda {
            body,
            parameters: Box::new(fetch.iter().map(|(id, (annotation, &ParamTyStub(tv)))| {
                kodept_inference::engine::Parameter {
                    tv,
                    name: (id.cast(), SymbolKind::Parameter),
                    bound: convert_annotation(annotation),
                }
            })),
        };
        let partial = DefaultTypeckContext.eval(item);
        modification.add_property(PartiallyTypechecked::from(partial));
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

#[derive(SystemParam)]
pub(super) struct TypeckVariable;

impl ParIterableSystem for TypeckVariable {
    type Iterable = HierarchicalQuery<
        'static,
        'static,
        Variable,
        Expression,
        Property<ResolvedTypeAnnotation>,
        Option<MutProperty<PartiallyTypechecked>>,
        Without<PartiallyTypechecked>,
    >;

    fn for_each<B: Buffer>(
        &self,
        mut modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
    ) -> impl TryReport {
        let (annotation, expr_fetch) = params;
        let (_, Some(mut expr_partial)) = expr_fetch.collect() else {
            return ();
        };

        let item = LangItem::Variable {
            bound: convert_annotation(annotation),
            body: expr_partial.take(),
        };
        let partial = DefaultTypeckContext.eval(item);
        modification.add_property(PartiallyTypechecked::from(partial));
    }
}
