use crate::report::Reporter;
use bevy_ecs::prelude::{In, IntoSystem};
use kodept_report::error::report::IntoSpannedReportMessage;

pub mod lint;
mod normalize;
pub mod report;
mod scope;

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
        ctx.register(Self::interaction().pipe(
            |In(result): In<Interacted<Self::Error>>, mut reporter: Reporter| match result {
                Ok(()) => {}
                Err(Skip::Skipped) => {}
                Err(Skip::Failed(e)) => reporter.report(e),
            },
        ));
    }
}
