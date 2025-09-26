use crate::utils::{wrap_system, Ctx, Disposable, Interaction, Try};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{
    Changed, Component, IntoScheduleConfigs, IntoSystem, Local, Populated, Query,
};
use kodept_report::traits::IntoSpannedReportMessage;
use std::borrow::Cow;
use tracing::info;

mod debug;
mod module;
mod rlt_linking;

pub use debug::*;
pub use module::SingleModuleWithBrackets;
pub use rlt_linking::RLTLinkLint;
use RunMode::{EachPass, Once, OnceRepeatable};

#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum RunMode {
    /// Runs lint only once
    Once,
    /// Runs lint once per enable
    OnceRepeatable,
    /// Runs lint on each pass
    EachPass,
}

#[derive(Debug, Component)]
pub struct LintDescriptor {
    pub enabled: bool,
    name: Cow<'static, str>,
    run_mode: RunMode,
}

impl LintDescriptor {
    pub const fn new(name: &'static str) -> Self {
        Self {
            enabled: true,
            name: Cow::Borrowed(name),
            run_mode: Once,
        }
    }

    pub fn disabled_by_default(self) -> Self {
        Self {
            enabled: false,
            ..self
        }
    }

    pub fn run_on_each_pass(self) -> Self {
        Self {
            run_mode: EachPass,
            ..self
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub trait Lint {
    #[allow(private_bounds)]
    type Result: Try<Output = (), Residual: IntoSpannedReportMessage + 'static> + 'static;

    fn descriptor() -> LintDescriptor;

    fn lint() -> impl IntoSystem<(), Self::Result, ()>;
}

struct LintGC(Entity);

impl<L: Lint> Interaction for L {
    fn name() -> Cow<'static, str> {
        Self::descriptor().name
    }

    fn install(ctx: &mut Ctx) -> impl Disposable + use<L> {
        let id = ctx.immediate_exclusive(|world| world.spawn(L::descriptor()).id());
        let config = wrap_system(Self::name(), L::lint()).run_if(
            move |lints: Query<&LintDescriptor>, mut has_run: Local<bool>| {
                let Ok(lint) = lints.get(id) else {
                    return false;
                };

                match (lint.enabled, lint.run_mode, *has_run) {
                    // Reset has_run if lint is disabled
                    (false, OnceRepeatable, true) => {
                        *has_run = false;
                        false
                    }
                    (false, _, _) => false,
                    (true, EachPass, _) => true,
                    (true, Once | OnceRepeatable, false) => {
                        *has_run = true;
                        true
                    }
                    (true, Once | OnceRepeatable, true) => false,
                }
            },
        );
        ctx.register(config);
        LintGC(id)
    }
}

impl Disposable for LintGC {
    fn dispose(&mut self, world: &mut bevy_ecs::world::World) {
        world.despawn(self.0);
    }
}

pub struct ShowLints;

impl Interaction for ShowLints {
    fn install(ctx: &mut Ctx) -> impl Disposable + use<> {
        ctx.register(wrap_system(
            Self::name(),
            |lints: Populated<&LintDescriptor, Changed<LintDescriptor>>| {
                let mut lint_names = String::new();
                let mut iter = lints.iter();

                if let Some(first) = iter.next() {
                    if first.enabled {
                        lint_names.push_str(&first.name);
                    }
                }
                for lint in iter {
                    if lint.enabled {
                        lint_names.push_str(", ");
                        lint_names.push_str(&lint.name)
                    }
                }

                info!("Enabled lints: [{lint_names}]");
            },
        ));
    }
}
