use crate::source::collection::Reporter;
use crate::utils::TryReport;
use kodept_ast::prelude::{
    ASTNode, ChildrenFetch, HierarchicalQuery, NarrowHierarchicalQuery, NodeId, NodeQueryData,
};
use kodept_ast::syntax_tree::children::{Family, HasChild};
use kodept_ast::syntax_tree::experimental::{Buffer, NodeModification};
use kodept_ecs::query::{QueryData, QueryFilter};
use kodept_ecs::system::{
    Commands, InMut, IntoSystem, Query, StaticSystemParam, SystemParam, SystemParamItem,
};
use kodept_report::prelude::MessageBehaviour;
use kodept_report::traits::IntoMessage;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::warn;

pub(super) type Modification<I, B> = NodeModification<<I as IterableSystemParam>::Node, B>;
pub(super) type Params<'w, 's, I> = <I as IterableSystemParam>::Target<'w, 's>;

pub(super) trait IterableSystem<Input = ()> {
    type Iterable: IterableSystemParam + 'static;

    fn for_each_with_input<B: Buffer>(
        &mut self,
        modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
        input: &mut Input,
    ) -> impl TryReport {
        _ = input;
        Self::for_each(self, modification, params)
    }

    fn for_each<B: Buffer>(
        &mut self,
        modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
    ) -> impl TryReport {
        _ = modification;
        _ = params;
    }
}

pub(super) trait ParIterableSystem<Input = ()> {
    type Iterable: IterableSystemParam + 'static;

    fn for_each_with_input<B: Buffer>(
        &self,
        modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
        input: &Input,
    ) -> impl TryReport {
        _ = input;
        Self::for_each(self, modification, params)
    }

    fn for_each<B: Buffer>(
        &self,
        modification: Modification<Self::Iterable, B>,
        params: Params<Self::Iterable>,
    ) -> impl TryReport {
        _ = modification;
        _ = params;
    }
}

impl<In, F: ParIterableSystem<In>> IterableSystem<In> for F {
    type Iterable = F::Iterable;

    #[inline]
    fn for_each_with_input<B: Buffer>(
        &mut self,
        modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
        input: &mut In,
    ) -> impl TryReport {
        F::for_each_with_input(self, modification, params, input)
    }
}

pub(super) trait IntoNodeSystem<Input> {
    fn system_with_input(input: Input) -> impl IntoSystem<(), (), ()>;

    fn system() -> impl IntoSystem<(), (), ()>
    where
        Input: Default,
    {
        Self::system_with_input(Input::default())
    }
}

pub(super) trait IntoParNodeSystem<Input> {
    fn par_system_with_input(input: Input) -> impl IntoSystem<(), (), ()>;

    fn par_system() -> impl IntoSystem<(), (), ()>
    where
        Input: Default,
    {
        Self::par_system_with_input(Input::default())
    }
}

#[inline]
fn handle_output<T: IntoMessage>(
    value: ControlFlow<T>,
    handler: impl FnOnce(T),
) -> ControlFlow<()> {
    match value {
        ControlFlow::Continue(()) => ControlFlow::Continue(()),
        ControlFlow::Break(error) => match error.behaviour() {
            MessageBehaviour::FailFast { reason } => {
                warn!("Stopping iteration: {reason}");
                handler(error);
                ControlFlow::Break(())
            }
            MessageBehaviour::Suppress => {
                handler(error);
                ControlFlow::Break(())
            }
        },
    }
}

impl<Input, F: SystemParam, I: IterableSystemParam> IntoNodeSystem<Input> for F
where
    Input: Send + Sync + 'static,
    F: 'static,
    I: 'static,
    for<'w, 's> SystemParamItem<'w, 's, F>: IterableSystem<Input, Iterable = I>,
    I::Node: ASTNode,
{
    fn system_with_input(input: Input) -> impl IntoSystem<(), (), ()> {
        let system = IntoSystem::into_system(
            |mut input: InMut<Input>,
             mut query: StaticSystemParam<I>,
             mut reporter: Reporter,
             mut extras: StaticSystemParam<F>,
             mut commands: Commands| {
                I::for_each(&mut query, move |id, rest| {
                    let modification = NodeModification::new(commands.reborrow(), id);
                    let output = IterableSystem::for_each_with_input(
                        &mut *extras,
                        modification,
                        rest,
                        &mut *input,
                    );
                    handle_output(output.branch(), |e| reporter.report(e))
                })
            },
        );

        let name = std::any::type_name::<F>();
        system.with_name(name).with_input(input)
    }
}

#[cfg(not(feature = "parallel"))]
impl<In, F: kodept_ecs::system::ReadOnlySystemParam, I: IterableSystemParam> IntoParNodeSystem<In>
    for F
where
    In: Send + Sync + 'static,
    F: 'static,
    I: 'static,
    for<'w, 's> F::Item<'w, 's>: ParIterableSystem<In, Iterable = I> + Sync,
    I::Node: ASTNode,
{
    fn par_system_with_input(input: In) -> impl IntoSystem<(), (), ()> {
        let system = IntoSystem::into_system(
            move |mut query: StaticSystemParam<I>,
                  mut reporter: Reporter,
                  extras: StaticSystemParam<F>,
                  mut commands: Commands| {
                let extras = &*extras;
                I::for_each(&mut query, |id, rest| {
                    let modification = NodeModification::new(&mut commands, id);
                    let output =
                        ParIterableSystem::for_each_with_input(extras, modification, rest, &input);
                    handle_output(output.branch(), |e| reporter.report(e))
                })
            },
        );

        let name = std::any::type_name::<F>();
        system.with_name(name)
    }
}

#[cfg(feature = "parallel")]
impl<In, F: kodept_ecs::system::ReadOnlySystemParam, I: IterableSystemParam> IntoParNodeSystem<In>
    for F
where
    In: Send + Sync + 'static,
    F: 'static,
    I: 'static,
    for<'w, 's> F::Item<'w, 's>: ParIterableSystem<In, Iterable = I> + Sync,
    I::Node: ASTNode,
{
    fn par_system_with_input(input: In) -> impl IntoSystem<(), (), ()> {
        let system = IntoSystem::into_system(
            move |mut query: StaticSystemParam<I>,
                  reporter: crate::source::collection::ParallelReporter,
                  extras: StaticSystemParam<F>,
                  commands: kodept_ecs::system::ParallelCommands| {
                let extras = &*extras;
                I::par_for_each(&mut query, |id, rest| {
                    let modification = NodeModification::new(&commands, id);
                    let output =
                        ParIterableSystem::for_each_with_input(extras, modification, rest, &input);
                    handle_output(output.branch(), |e| reporter.report(e))
                })
            },
        );

        let name = std::any::type_name::<F>();
        system.with_name(Cow::Borrowed(name))
    }
}

pub(super) trait IterableSystemParam: SystemParam {
    type Target<'w, 's>;
    type Node;

    fn for_each<'s>(
        this: &mut Self::Item<'_, 's>,
        on_each: impl FnMut(NodeId<Self::Node>, Self::Target<'_, 's>) -> ControlFlow<()>,
    );

    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    fn par_for_each<'s>(
        this: &mut Self::Item<'_, 's>,
        on_each: impl Fn(NodeId<Self::Node>, Self::Target<'_, 's>) -> ControlFlow<()>
        + Send
        + Sync
        + Clone,
    );
}

pub(super) trait SplitFirst {
    type Head;
    type Tail;

    fn split(self) -> (Self::Head, Self::Tail);
}

pub(super) type StaticQuery<T, F = ()> = Query<'static, 'static, T, F>;

impl<T, F, Node> IterableSystemParam for Query<'_, '_, T, F>
where
    T: QueryData + SplitFirst<Head = NodeId<Node>, Tail: NodeQueryData<Node>> + 'static,
    F: QueryFilter + 'static,
    for<'w, 's> T::Item<'w, 's>: SplitFirst<Head = NodeId<Node>>,
{
    type Target<'w, 's> = <T::Item<'w, 's> as SplitFirst>::Tail;
    type Node = Node;

    #[inline]
    fn for_each<'s>(
        this: &mut Self::Item<'_, 's>,
        mut on_each: impl FnMut(NodeId<Self::Node>, Self::Target<'_, 's>) -> ControlFlow<()>,
    ) {
        _ = this.iter_mut().try_for_each(|it| {
            let (id, rest) = it.split();
            on_each(id, rest)
        });
    }

    #[inline]
    fn par_for_each<'s>(
        this: &mut Self::Item<'_, 's>,
        on_each: impl Fn(NodeId<Self::Node>, Self::Target<'_, 's>) -> ControlFlow<()>
        + Send
        + Sync
        + Clone,
    ) {
        let should_stop = AtomicBool::new(false);
        this.par_iter_mut().for_each(|it| {
            if should_stop.load(Ordering::Acquire) {
                return;
            }
            let (id, rest) = it.split();
            if on_each(id, rest).is_break() {
                should_stop.store(true, Ordering::Release);
            }
        })
    }
}

impl<P, T, PD, CD, F> IterableSystemParam for HierarchicalQuery<'_, '_, P, T, PD, CD, F>
where
    P: ASTNode + Family<T>,
    PD: NodeQueryData<P>,
    CD: QueryData,
    F: QueryFilter,
{
    type Target<'w, 's> = (
        PD::Item<'w, 's>,
        ChildrenFetch<'w, 's, CD::ReadOnly, P, T, NodeId>,
    );
    type Node = P;

    #[inline]
    fn for_each<'s>(
        this: &mut Self::Item<'_, 's>,
        mut on_each: impl FnMut(NodeId<Self::Node>, Self::Target<'_, 's>) -> ControlFlow<()>,
    ) {
        _ = this
            .iter_by_layers()
            .try_for_each(|(id, parent, children)| on_each(id, (parent, children)));
    }

    #[inline]
    fn par_for_each<'s>(
        this: &mut Self::Item<'_, 's>,
        on_each: impl Fn(NodeId<Self::Node>, Self::Target<'_, 's>) -> ControlFlow<()>
        + Send
        + Sync
        + Clone,
    ) {
        let should_stop = AtomicBool::new(false);
        this.par_iter_by_layers(|id, parent, children| {
            if should_stop.load(Ordering::Acquire) {
                return;
            }
            if on_each(id, (parent, children)).is_break() {
                should_stop.store(true, Ordering::Release);
            }
        });
    }
}

impl<P, C, T, PD, CD, F> IterableSystemParam for NarrowHierarchicalQuery<'_, '_, P, C, T, PD, CD, F>
where
    P: HasChild<C, T>,
    C: ASTNode,
    PD: NodeQueryData<P>,
    CD: NodeQueryData<C>,
    F: QueryFilter,
{
    type Target<'w, 's> = (
        PD::Item<'w, 's>,
        ChildrenFetch<'w, 's, CD::ReadOnly, P, T, NodeId<C>>,
    );
    type Node = P;

    #[inline]
    fn for_each<'s>(
        this: &mut Self::Item<'_, 's>,
        mut on_each: impl FnMut(NodeId<Self::Node>, Self::Target<'_, 's>) -> ControlFlow<()>,
    ) {
        _ = this
            .iter_by_layers()
            .try_for_each(|(id, parent, children)| on_each(id, (parent, children)));
    }

    #[inline]
    fn par_for_each<'s>(
        this: &mut Self::Item<'_, 's>,
        on_each: impl Fn(NodeId<Self::Node>, Self::Target<'_, 's>) -> ControlFlow<()>
        + Send
        + Sync
        + Clone,
    ) {
        let should_stop = AtomicBool::new(false);
        this.par_iter_by_layers(|id, parent, children| {
            if should_stop.load(Ordering::Acquire) {
                return;
            }
            if on_each(id, (parent, children)).is_break() {
                should_stop.store(true, Ordering::Release);
            }
        });
    }
}

macro_rules! impl_extract_on_tuple {
    ($($Ts:ident$(,)?)* => [$self:ident]$split:expr) => {
        impl<T, $($Ts, )*> SplitFirst for (NodeId<T>, $($Ts, )*) {
            type Head = NodeId<T>;
            type Tail = ($($Ts, )*);

            #[inline(always)]
            fn split($self) -> (Self::Head, Self::Tail) {
                $split
            }
        }
    };
}

impl_extract_on_tuple!(=> [self] (self.0, ()));
impl_extract_on_tuple!(A => [self] (self.0, (self.1, )));
impl_extract_on_tuple!(A, B => [self] (self.0, (self.1, self.2)));
impl_extract_on_tuple!(A, B, C => [self] (self.0, (self.1, self.2, self.3)));
impl_extract_on_tuple!(A, B, C, D => [self] (self.0, (self.1, self.2, self.3, self.4)));
impl_extract_on_tuple!(A, B, C, D, E => [self] (self.0, (self.1, self.2, self.3, self.4, self.5)));
impl_extract_on_tuple!(A, B, C, D, E, F => [self] (self.0, (self.1, self.2, self.3, self.4, self.5, self.6)));
impl_extract_on_tuple!(A, B, C, D, E, F, G => [self] (self.0, (self.1, self.2, self.3, self.4, self.5, self.6, self.7)));
impl_extract_on_tuple!(A, B, C, D, E, F, G, H => [self] (self.0, (self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8)));
impl_extract_on_tuple!(A, B, C, D, E, F, G, H, I => [self] (self.0, (self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9)));
impl_extract_on_tuple!(A, B, C, D, E, F, G, H, I, J => [self] (self.0, (self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9, self.10)));
