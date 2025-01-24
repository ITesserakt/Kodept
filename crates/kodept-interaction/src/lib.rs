use crate::report::Reporter;
use bevy_ecs::prelude::{In, IntoSystem};
use tracing::trace;
use kodept_report::error::report::IntoSpannedReportMessage;

pub mod lint;
mod normalize;
pub mod report;
pub mod scope;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Skip<E> {
    Skipped,
    Failed(E),
}

pub type Interacted<E> = Result<(), Skip<E>>;
type Ctx<'w> = kodept_ast::interaction::Interaction<'w>;

fn skip<E>() -> Interacted<E> {
    Err(Skip::Skipped)
}

fn fail<E>(error: E) -> Interacted<E> {
    Err(Skip::Failed(error))
}

fn done<E>() -> Interacted<E> {
    Ok(())
}

pub trait Interaction<M = ()> {
    type Error: IntoSpannedReportMessage + 'static;

    fn interaction() -> impl IntoSystem<(), Interacted<Self::Error>, M>;

    fn install(ctx: &mut Ctx) {
        let original_system = Self::interaction();
        let original_system_name = original_system.system_type_id();
        ctx.register(original_system.pipe(
            move |In(result): In<Interacted<Self::Error>>, reporter: Reporter| match result {
                Ok(()) => trace!("System {original_system_name:?} completed"),
                Err(Skip::Skipped) => trace!("System {original_system_name:?} skipped"),
                Err(Skip::Failed(e)) => reporter.report(e),
            },
        ));
    }
}
