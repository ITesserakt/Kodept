use crate::source::collection::Reporter;
use bevy_ecs::prelude::*;
use kodept_core::try_port::Try;
use kodept_frontend::prelude::ExtractReports;
use std::ops::ControlFlow;
use kodept_report::prelude::IntoSpannedReportMessage;

pub trait ReportSystemEx<Out, SystemMarker, ExtractMarker> {
    fn extract_reports(self) -> impl IntoSystem<(), (), ()>;
}

impl<SystemMarker, ExtractMarker, Out, T: IntoSystem<(), Out, SystemMarker>>
    ReportSystemEx<Out, SystemMarker, ExtractMarker> for T
where
    Out: Try<Output = ()> + 'static,
    Out::Residual: ExtractReports<ExtractMarker>,
{
    fn extract_reports(self) -> impl IntoSystem<(), (), ()> {
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

#[derive(Debug)]
pub struct ForwardReport<T>(pub T);

impl<T: IntoSpannedReportMessage> Try for ForwardReport<T> {
    type Output = ();
    type Residual = T;

    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Break(self.0)
    }
} 
