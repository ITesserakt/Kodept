#![allow(dead_code)]

use kodept_report::traits::IntoSpannedReportMessage;
use std::borrow::Cow;

pub mod lint;
mod normalize;
mod phase;
mod report;
mod scope;
mod symbol;

pub mod prelude {
    pub use super::phase::Phases;
    pub use super::report::install_reporting_support;
    pub use super::scope::builder::ScopeBuildingPass;
    pub use super::scope::references::ReferenceResolverPass;
    pub use super::symbol::interaction::{DuplicatedSymbolError, ExtractSymbolsPass};
}

pub type Result<E> = std::result::Result<(), E>;
type Ctx<'w> = kodept_ast::interaction::Interaction<'w>;

fn fail<E>(error: E) -> Result<E> {
    Err(error)
}

fn done<E>() -> Result<E> {
    Ok(())
}

pub(crate) mod wrapper {
    use crate::report::Reporter;
    use crate::Interaction;
    use bevy_ecs::prelude::*;
    use kodept_report::prelude::{IntoSpannedReportMessage, MessageBehaviour};
    use tracing::trace;

    pub(crate) trait InteractionExt: Interaction {
        fn wrap_system<S, M>(system: S) -> impl IntoSystem<(), Result, ()>
        where
            S: IntoSystem<(), crate::Result<Self::Error>, M>;
    }

    impl<I: Interaction> InteractionExt for I {
        fn wrap_system<S, M>(system: S) -> impl IntoSystem<(), Result, ()>
        where
            S: IntoSystem<(), crate::Result<Self::Error>, M>,
        {
            let name = I::name();
            let id = system.system_type_id();
            let system = system.pipe(
                move |In(result): In<crate::Result<Self::Error>>, reporter: Reporter| match result {
                    Ok(()) => {
                        trace!("System {name}#{id:?} completed");
                        Ok(())
                    }
                    Err(e) => {
                        let behaviour = e.behaviour();
                        reporter.report(e);
                        match behaviour {
                            MessageBehaviour::FailFast { reason } => Err(reason.into()),
                            MessageBehaviour::Suppress => Ok(()),
                        }
                    }
                },
            );
            IntoSystem::into_system(system)
        }
    }
}

pub trait Interaction: Sized {
    type Error: IntoSpannedReportMessage + 'static;

    fn name() -> Cow<'static, str> {
        let name = std::any::type_name::<Self>();
        Cow::Borrowed(name.rsplit_once("::").map_or(name, |it| it.1))
    }

    fn install(ctx: &mut Ctx);
}
