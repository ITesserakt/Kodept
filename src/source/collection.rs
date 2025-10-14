use std::ops::ControlFlow;
use bevy_ecs::prelude::{In, IntoSystem};
use kodept_core::try_port::Try;
use kodept_frontend::prelude::{ExtractReports};
use crate::source::loaded::SourceImpl;

pub type Sources = kodept_frontend::prelude::SourceFiles<SourceImpl>;
pub type SourceView = kodept_frontend::prelude::SourceView<SourceImpl>;
pub type Reporter<'w, 's> = kodept_frontend::engine::reporter::Reporter<'w, 's, SourceImpl>;

pub trait SystemExt<Out, SystemMarker, ExtractMarker> {
    fn report_errors(self) -> impl IntoSystem<(), (), ()>;
}

impl<SystemMarker, ExtractMarker, Out, T: IntoSystem<(), Out, SystemMarker>>
    SystemExt<Out, SystemMarker, ExtractMarker> for T
where
    Out: Try<Output = (), Residual: ExtractReports<ExtractMarker>> + 'static,
{
    fn report_errors(self) -> impl IntoSystem<(), (), ()>
    {
        IntoSystem::into_system(
            self.pipe(
                |In(output): In<Out>, mut reporter: Reporter| match output.branch() {
                    ControlFlow::Continue(_) => {}
                    ControlFlow::Break(e) => {
                        e.extract_reports(&mut reporter);
                    }
                },
            ),
        )
    }
}
