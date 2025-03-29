#![allow(dead_code)]

use crate::wrapper::InteractionWrapper;
use kodept_ast::prelude::NodeId;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_core::structure::Located;
use kodept_report::message::SpannedError;
use kodept_report::traits::IntoSpannedReportMessage;

pub mod lint;
mod normalize;
mod report;
mod scope;
mod symbol;

pub mod prelude {
    pub use super::scope::builder::ScopeBuildingPass;
    pub use super::scope::references::ReferenceResolver;

    pub use super::symbol::interaction::{DuplicatedSymbolError, ExtractSymbols};

    pub use super::report::ASTExt;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Skip<E> {
    Skipped,
    Failed(E),
}

pub type Result<E> = std::result::Result<(), Skip<E>>;
type Ctx<'w> = kodept_ast::interaction::Interaction<'w>;

fn skip<E>() -> Result<E> {
    Err(Skip::Skipped)
}

fn fail<E>(error: E) -> Result<E> {
    Err(Skip::Failed(error))
}

fn done<E>() -> Result<E> {
    Ok(())
}

impl<E> From<E> for Skip<E> {
    fn from(value: E) -> Self {
        Self::Failed(value)
    }
}

pub mod wrapper {
    use crate::report::Reporter;
    use crate::Skip;
    use bevy_ecs::prelude::{In, IntoSystem, IntoScheduleConfigs};
    use kodept_report::traits::IntoSpannedReportMessage;
    use std::marker::PhantomData;
    use bevy_ecs::schedule::ScheduleConfigs;
    use bevy_ecs::system::ScheduleSystem;
    use tracing::trace;

    pub struct InteractionWrapper<E>(PhantomData<E>, ScheduleConfigs<ScheduleSystem>);

    impl<E> InteractionWrapper<E> {
        pub(crate) fn wrap<M, S>(system: S) -> Self
        where
            S: IntoSystem<(), crate::Result<E>, M>,
            E: IntoSpannedReportMessage + 'static,
        {
            let name = system.system_type_id();
            let piped_system = system.pipe(
                move |In(result): In<crate::Result<E>>, reporter: Reporter| match result {
                    Ok(()) => trace!("System {name:?} completed"),
                    Err(Skip::Skipped) => trace!("System {name:?} skipped"),
                    Err(Skip::Failed(e)) => reporter.report(e),
                },
            );
            Self(PhantomData, IntoScheduleConfigs::into_configs(piped_system))
        }

        pub(crate) fn from_configs(configs: impl IntoScheduleConfigs<ScheduleSystem, ()>) -> Self {
            InteractionWrapper(PhantomData, configs.into_configs())
        }

        pub fn unwrap(self) -> impl IntoScheduleConfigs<ScheduleSystem, ()> {
            self.1
        }
    }
}

pub trait Interaction<M = ()> {
    type Error: IntoSpannedReportMessage + 'static;

    fn interaction() -> InteractionWrapper<Self::Error>;

    fn install(ctx: &mut Ctx) {
        ctx.register(Self::interaction().unwrap());
    }
}

pub trait SpannedErrorExt<E> {
    fn for_node(inner: E, node_id: NodeId, syntax: &SyntaxResolver) -> Self;
}

impl<E: std::error::Error> SpannedErrorExt<E> for SpannedError<E> {
    fn for_node(inner: E, node_id: NodeId, syntax: &SyntaxResolver) -> Self {
        let variant = syntax.try_get_unknown(node_id).unwrap();
        let point = variant.location();
        Self::new(inner, point)
    }
}
