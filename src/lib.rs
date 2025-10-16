pub mod loader;
pub mod source;

pub mod utils {
    use crate::source::collection::Reporter;
    use kodept_core::try_port::Try;
    use kodept_frontend::prelude::ExtractReports;
    use std::ops::ControlFlow;
    use bevy_ecs::prelude::{In, IntoSystem};

    pub trait ReportSystemEx<Out, SystemMarker, ExtractMarker> {
        fn report_errors(self) -> impl IntoSystem<(), (), ()>;
    }

    impl<SystemMarker, ExtractMarker, Out, T: IntoSystem<(), Out, SystemMarker>>
        ReportSystemEx<Out, SystemMarker, ExtractMarker> for T
    where
        Out: Try<Output = (), Residual: ExtractReports<ExtractMarker, Output = ()>> + 'static,
    {
        fn report_errors(self) -> impl IntoSystem<(), (), ()> {
            IntoSystem::into_system(self.pipe(|In(output): In<Out>, mut reporter: Reporter| {
                match output.branch() {
                    ControlFlow::Continue(_) => {}
                    ControlFlow::Break(e) => {
                        e.extract_reports(&mut reporter);
                    }
                }
            }))
        }
    }
}
