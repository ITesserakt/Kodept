use codespan_reporting::diagnostic::{Diagnostic, Label};
use extend::ext;
use itertools::Itertools;
use kodept_parse::error::ParseError;

#[ext]
pub impl<'t> ParseError<'t> {
    fn to_diagnostic(self) -> Diagnostic<()> {
        let exp_msg = self
            .expected
            .into_iter()
            .map(|it| format!("`{it}`"))
            .join(" or ");

        Diagnostic::error()
            .with_code("SE001")
            .with_message(format!("Expected {}, got `{}`", exp_msg, self.actual))
            .with_labels(vec![Label::primary((), self.location.in_code.as_range())])
    }
}
