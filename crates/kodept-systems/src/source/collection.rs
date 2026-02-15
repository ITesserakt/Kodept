use crate::source::loaded::SourceImpl;

pub type Sources = kodept_frontend::prelude::SourceFiles<SourceImpl>;
pub type SourceView = kodept_frontend::prelude::SourceView<SourceImpl>;
pub type Reporter<'w, 's> = kodept_frontend::engine::reporter::Reporter<'w, 's, SourceImpl>;
#[cfg(feature = "parallel")]
pub type ParallelReporter<'w, 's> =
    kodept_frontend::engine::reporter::ParallelReporter<'w, 's, SourceImpl>;
