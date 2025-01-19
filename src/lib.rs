pub mod code_source;
pub mod codespan_settings;
pub mod common_iter;
pub mod loader;
pub mod read_code_source;
// pub mod steps;
pub mod context;
pub mod hlist;
pub mod utils;

pub mod source_files {
    use std::ops::Deref;
    use std::sync::Arc;
    use kodept_frontend::external::Resource;
    use crate::read_code_source::SourceImpl;

    #[derive(Resource)]
    pub struct Sources(pub Arc<SourceFiles>);
    #[derive(Resource)]
    pub struct SourcesBuilder(pub SourceFiles);

    pub type SourceFiles = kodept_frontend::prelude::SourceFiles<SourceImpl>;
    pub type SourceView = kodept_frontend::prelude::SourceView<SourceImpl>;

    impl Deref for Sources {
        type Target = SourceFiles;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
