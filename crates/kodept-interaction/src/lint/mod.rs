use crate::report::Reporter;
use crate::{Ctx, Interacted, Interaction, Skip};
use bevy_ecs::prelude::{Component, In, IntoSystem, IntoSystemConfigs, Query, World};
use bevy_ecs::system::SystemId;
use kodept_report::error::report::IntoSpannedReportMessage;
use std::borrow::Cow;
use std::sync::OnceLock;
use tracing::trace;

pub mod module;
pub mod rlt_linking;

#[derive(Debug, Component)]
pub struct LintDescriptor {
    enabled: bool,
    name: Cow<'static, str>,
    system_id: OnceLock<SystemId>,
}

impl LintDescriptor {
    pub const fn new(name: &'static str) -> Self {
        Self {
            enabled: true,
            name: Cow::Borrowed(name),
            system_id: OnceLock::new(),
        }
    }

    pub fn set_system(&mut self, id: SystemId) {
        _ = self.system_id.set(id);
    }
}

pub trait Lint {
    type Error: IntoSpannedReportMessage + 'static;

    fn descriptor() -> LintDescriptor;

    fn lint() -> impl IntoSystem<(), Interacted<Self::Error>, ()>;
}

impl<L: Lint> Interaction for L {
    type Error = L::Error;

    fn interaction() -> impl IntoSystem<(), Interacted<Self::Error>, ()> {
        L::lint()
    }

    fn install(ctx: &mut Ctx) {
        let id = ctx.immediate(|world: &mut World| world.spawn(L::descriptor()).id());
        let original_system = L::interaction();
        let original_system_name = original_system.system_type_id();
        let system = original_system.pipe(
            move |In(result): In<Interacted<Self::Error>>, mut reporter: Reporter| match result {
                Ok(_) => trace!("System {original_system_name:?} completed"),
                Err(Skip::Skipped) => trace!("System {original_system_name:?} skipped"),
                Err(Skip::Failed(e)) => reporter.report(e),
            },
        );

        ctx.register(
            system.run_if(move |lints: Query<&LintDescriptor>| {
                lints.get(id).is_ok_and(|it| it.enabled)
            }),
        );
    }
}

#[macro_export]
macro_rules! define_lint {
    ($vis:vis $this:ident[$name:literal] $body:expr) => {
        $vis struct $this;

        impl $crate::lint::Lint for $this {
            fn descriptor() -> LintDescriptor {
                $crate::lint::LintDescriptor::new($name)
            }

            fn interaction() -> impl IntoSystem<(), (), ()> {
                bevy_ecs::prelude::IntoSystem::into_system($body)
            }
        }
    };
}
