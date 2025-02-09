use kodept_ast::utils::Skip;
use kodept_ast::utils::Skip::Skipped;
use kodept_ast::{FileDecl};
use kodept_core::code_point::CodePoint;
use kodept_rlt::prelude::{File, Module};
use kodept_core::structure::Located;
use std::convert::Infallible;
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, Label, Severity};
use crate::macros::{Context, Macro, VisitGuard};

/// This lint suggests to replace
/// ```kodept
/// module Name {
/// }
/// ```
/// to
/// ```kodept
/// module Name =>
/// ```
/// if there is only one module in file
pub struct SingleModuleBracketsLint;

struct SingleModuleBracketsMessage {
    brackets_pos: [CodePoint; 2],
}

impl IntoSpannedReportMessage for SingleModuleBracketsMessage {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Warning)
            .with_message("Consider replacing brackets with single `=>`")
            .with_label(Label::primary("replace with `=>`", self.brackets_pos[0]))
            .with_label(Label::primary("remove", self.brackets_pos[1]))
    }
}

impl Macro for SingleModuleBracketsLint {
    type Error = Infallible;
    type Node = FileDecl;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Result<(), Skip<Self::Error>> {
        let id = guard.allow_last().ok_or(Skipped)?;
        let rlt: &File = ctx.rlt.get(id).ok_or(Skipped)?;
        
        // do not check real ast state, and instead check only rlt structure
        // ast may contain fake modules
        
        if let [module] = &rlt.0.as_ref() {
            if let Module::Ordinary { lbrace, rbrace, .. } = module {
                ctx.report(SingleModuleBracketsMessage {
                    brackets_pos: [lbrace.location(), rbrace.location()],
                });
            }
        }

        Ok(())
    }
}
