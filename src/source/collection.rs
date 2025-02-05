use crate::source::loaded::SourceImpl;

pub type Sources = kodept_frontend::prelude::SourceFiles<SourceImpl>;
pub type SourceView = kodept_frontend::prelude::SourceView<SourceImpl>;

const _: () = {
    use codespan_reporting::files::Files;

    const fn assert_impls<'a, T: Files<'a>>() {}

    assert_impls::<Sources>();
    assert_impls::<SourceView>();
};
