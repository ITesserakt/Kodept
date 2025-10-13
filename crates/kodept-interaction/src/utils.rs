use crate::report::Reporter;
use bevy_ecs::message::{Message, MessageReader};
use bevy_ecs::prelude::Commands;
use bevy_ecs::{
    system::{In, IntoSystem},
    world::World,
};
use kodept_ast::interaction::ObserverEntity;
use kodept_report::traits::{IntoSpannedReportMessage, MessageBehaviour};
use std::fmt::{Display, Formatter};
use std::{any::TypeId, borrow::Cow, ops::ControlFlow};

pub(crate) type Ctx<'w> = kodept_ast::interaction::Interaction<'w>;

/// Port of stdlib's `Try` into stable Rust
pub(crate) trait Try {
    type Output;
    type Residual;

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

impl<T, E> Try for Result<T, E> {
    type Output = T;
    type Residual = E;

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(x) => ControlFlow::Continue(x),
            Err(x) => ControlFlow::Break(x),
        }
    }
}

pub(crate) trait FnOutput {
    type Output;
}

impl<T> FnOutput for fn() -> T {
    type Output = T;
}

type Never = <fn() -> ! as FnOutput>::Output;

impl Try for () {
    type Output = ();
    type Residual = Never;

    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(())
    }
}

impl<T> Try for Option<T> {
    type Output = T;
    type Residual = Option<Never>;

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Some(x) => ControlFlow::Continue(x),
            None => ControlFlow::Break(None),
        }
    }
}

impl<C, B> Try for ControlFlow<B, C> {
    type Output = C;
    type Residual = B;

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        self
    }
}

#[derive(Debug, Message)]
pub struct SystemCompletionEvent {
    pub name: Cow<'static, str>,
    pub id: TypeId,
    pub fail_reason: Option<Cow<'static, str>>,
}

#[derive(Debug)]
struct FailFastReason(Cow<'static, str>);

pub fn install_system_completion_introspection_support(
    ctx: &mut Ctx,
    mut f: impl FnMut(&SystemCompletionEvent) + Send + Sync + 'static,
) {
    ctx.register_message::<SystemCompletionEvent>();

    ctx.register(move |mut reader: MessageReader<SystemCompletionEvent>| {
        reader.read().for_each(&mut f);
    });
}

/// Transforms a system that returns a result of some error [`E`] to a system that returns bevy's result according to the [behaviour](kodept_rlt::traits::MessageBehaviour) of error [`E`]
/// Additionally sends an event about completion status
pub(crate) fn wrap_system<S, T, M>(
    name: Cow<'static, str>,
    system: S,
) -> impl IntoSystem<(), (), ()>
where
    S: IntoSystem<(), T, M>,
    T: Try<Output = ()> + 'static,
    T::Residual: IntoSpannedReportMessage + 'static,
{
    let id = system.system_type_id();
    let system = system.pipe(
        move |In(result): In<T>, reporter: Reporter, mut commands: Commands| match result.branch() {
            ControlFlow::Continue(()) => {
                commands.write_message(SystemCompletionEvent {
                    name: name.clone(),
                    id,
                    fail_reason: None,
                });
            }
            ControlFlow::Break(error) => {
                let behaviour = error.behaviour();
                reporter.report(error);
                match behaviour {
                    MessageBehaviour::FailFast { reason } => {
                        commands.write_message(SystemCompletionEvent {
                            name: name.clone(),
                            id,
                            fail_reason: Some(reason.clone()),
                        });
                    }
                    MessageBehaviour::Suppress => {
                        commands.write_message(SystemCompletionEvent {
                            name: name.clone(),
                            id,
                            fail_reason: None,
                        });
                    }
                }
            }
        },
    );

    IntoSystem::into_system(system)
}

pub trait Disposable {
    fn dispose(&mut self, world: &mut World);
}

pub trait Interaction {
    fn name() -> Cow<'static, str> {
        let name = std::any::type_name::<Self>();
        Cow::Borrowed(name.rsplit_once("::").map_or(name, |it| it.1))
    }

    #[must_use]
    fn install(ctx: &mut Ctx) -> impl Disposable + use<Self>;
}

macro_rules! tuple_please {
    () => {
        impl Disposable for () {
            #[inline(always)]
            fn dispose(&mut self, _: &mut World) {}
        }
    };
    ($($ty:ident as $ty_access:tt$(,)?)*) => {
        impl <$($ty: Disposable, )*> Disposable for ($($ty, )*) {
            #[inline]
            fn dispose(&mut self, world: &mut World) {
                $(
                    Disposable::dispose(&mut self.$ty_access, world);
                )*
            }
        }
    };
}

tuple_please!();
tuple_please!(A as 0);
tuple_please!(A as 0, B as 1);
tuple_please!(A as 0, B as 1, C as 2);
tuple_please!(A as 0, B as 1, C as 2, D as 3);

impl Display for FailFastReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for FailFastReason {}

impl<T: Disposable, const N: usize> Disposable for [T; N] {
    fn dispose(&mut self, world: &mut World) {
        for item in self {
            item.dispose(world);
        }
    }
}

impl Disposable for ObserverEntity {
    fn dispose(&mut self, world: &mut World) {
        world.despawn(self.0);
    }
}
