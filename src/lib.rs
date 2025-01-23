pub mod loader;
pub mod source;

pub mod report {
    use crate::source::loaded::SourceImpl;

    pub type Reports = kodept_frontend::prelude::Reports<SourceImpl>;
    pub type GlobalReports = kodept_frontend::prelude::GlobalReports<SourceImpl>;
}
