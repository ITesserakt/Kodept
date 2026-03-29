use kodept_ecs::schedule::IntoScheduleConfigs;
use kodept_ecs::system::{In, IntoSystem, ScheduleSystem, System, SystemInput};
use kodept_frontend::engine::reporter::Reporter;
use kodept_report::prelude::IntoMessage;
use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::Debug;
use std::ops::ControlFlow;
use tracing::trace;

pub(super) trait TryReport<T = ()> {
    type Output: IntoMessage;

    fn branch(self) -> ControlFlow<Self::Output, T>;
}

impl<T> TryReport<T> for T {
    type Output = Infallible;

    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Output, T> {
        ControlFlow::Continue(self)
    }
}

impl<T, R: IntoMessage> TryReport<T> for Result<T, R> {
    type Output = R;

    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Output, T> {
        match self {
            Ok(x) => ControlFlow::Continue(x),
            Err(e) => ControlFlow::Break(e),
        }
    }
}

impl<T, E: IntoMessage> TryReport<T> for ControlFlow<E, T> {
    type Output = E;

    #[inline(always)]
    fn branch(self) -> ControlFlow<E, T> {
        self
    }
}

pub trait ReportSystemEx<In, Out, SystemMarker>
where
    In: SystemInput,
{
    fn extract_reports(self) -> impl IntoSystem<In, (), ()>;
}

pub trait LogSystemEx<In, Out, SystemMarker>
where
    In: SystemInput,
{
    fn trace_completion(self) -> impl IntoSystem<In, Out, ()>;

    fn trace_completion_with_name(
        self,
        name: impl Into<Cow<'static, str>>,
    ) -> impl IntoSystem<In, Out, ()>;
}

pub trait LogSystemSetEx<Marker> {
    fn trace_completion(self) -> impl IntoScheduleConfigs<ScheduleSystem, ()>;
}

impl<SystemMarker, Input, Out, T: IntoSystem<Input, Out, SystemMarker>>
    ReportSystemEx<Input, Out, SystemMarker> for T
where
    Out: TryReport + 'static,
    Input: SystemInput,
{
    #[track_caller]
    fn extract_reports(self) -> impl IntoSystem<Input, (), ()> {
        IntoSystem::into_system(self.pipe(|In(output): In<Out>, mut reporter: Reporter| {
            match output.branch() {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(error) => reporter.report(error),
            }
        }))
    }
}

impl<In, Out, SystemMarker, T> LogSystemEx<In, Out, SystemMarker> for T
where
    T: IntoSystem<In, Out, SystemMarker>,
    In: SystemInput,
    Out: Debug,
{
    #[track_caller]
    fn trace_completion(self) -> impl IntoSystem<In, Out, ()> {
        let system = IntoSystem::into_system(self);
        let name = system.name();
        system.trace_completion_with_name(name)
    }

    #[track_caller]
    fn trace_completion_with_name(
        self,
        name: impl Into<Cow<'static, str>>,
    ) -> impl IntoSystem<In, Out, ()> {
        let id = self.system_type_id();
        let name = name.into();
        let system = self.map(move |out| {
            trace!(system_id=?id, output=?out, "System `{name}` completed");
            out
        });
        IntoSystem::into_system(system)
    }
}

macro_rules! impl_log_system_set_ex {
    ($([$t:ident, $m:ident, $field:tt]$(,)?)+) => {
        impl<$($t, $m, )+> LogSystemSetEx<($($m, )+)> for ($($t, )+)
        where
            $($t: IntoSystem<(), (), $m>,)+
        {
            #[track_caller]
            fn trace_completion(self) -> impl IntoScheduleConfigs<ScheduleSystem, ()> {
                IntoScheduleConfigs::into_configs(
                    ($({self.$field}.trace_completion(),)+)
                )
            }
        }
    };
}

impl_log_system_set_ex! {
    [T1, M1, 0],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
    [T8, M8, 7],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
    [T8, M8, 7],
    [T9, M9, 8],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
    [T8, M8, 7],
    [T9, M9, 8],
    [T10, M10, 9],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
    [T8, M8, 7],
    [T9, M9, 8],
    [T10, M10, 9],
    [T11, M11, 10],
}

pub type ForwardReport<T> = Result<(), T>;

#[inline(always)]
pub fn forward<T>(report: T) -> ForwardReport<T>
where
    T: IntoMessage,
{
    Err(report)
}
