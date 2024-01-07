use std::convert::Infallible;

use codespan_reporting::diagnostic::Severity;

use kodept_ast::visitor::visit_side::{VisitGuard, VisitSide};
use kodept_ast::visitor::TraversingResult;
use kodept_ast::{EnumDeclaration, StructDeclaration};
use kodept_core::impl_named;
use kodept_core::structure::rlt::Enum;
use kodept_core::structure::{rlt, Located};

use crate::analyzer::Analyzer;
use crate::error::report::ReportMessage;
use crate::traits::Context;
use crate::warn_about_broken_rlt;

#[derive(Debug)]
pub struct StructAnalyzer;
#[derive(Debug)]
pub struct EnumAnalyzer;

impl_named!(StructAnalyzer);
impl_named!(EnumAnalyzer);

pub struct EmptyParametersList;
pub struct EmptyBody;

impl From<EmptyParametersList> for ReportMessage {
    fn from(_value: EmptyParametersList) -> Self {
        ReportMessage::new(
            Severity::Warning,
            "SE001",
            "Remove empty parentheses or add some parameters".to_string(),
        )
    }
}

impl From<EmptyBody> for ReportMessage {
    fn from(_value: EmptyBody) -> Self {
        ReportMessage::new(
            Severity::Warning,
            "SE001",
            "Body does not contain any items, consider to remove brackets".to_string(),
        )
    }
}

impl Analyzer for StructAnalyzer {
    type Error = Infallible;
    type Node = StructDeclaration;

    fn analyze<'n, 'c, C: Context<'c>>(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut C,
    ) -> TraversingResult<Self::Error> {
        let node = guard.allow_only(VisitSide::Entering)?;
        let rlt: Option<&rlt::Struct> = context.access(&*node);
        let tree = context.tree();

        // if node.parameters(&tree, &*token).is_empty() {
        //     match rlt.map(|it| it.parameters.as_ref()) {
        //         None => {
        //             warn_about_broken_rlt::<rlt::Struct>();
        //             context.add_report(vec![], EmptyParametersList);
        //         }
        //         Some(None) => {}
        //         Some(Some(params)) => context.add_report(
        //             vec![params.left.location(), params.right.location()],
        //             EmptyParametersList,
        //         ),
        //     };
        // }
        //
        // if node.contents(&tree, &*token).is_empty() {
        //     match rlt.map(|it| it.body.as_ref()) {
        //         None => {
        //             warn_about_broken_rlt::<rlt::Struct>();
        //             context.add_report(vec![], EmptyParametersList);
        //         }
        //         Some(None) => {}
        //         Some(Some(rest)) => {
        //             context.add_report(vec![rest.left.location(), rest.right.location()], EmptyBody)
        //         }
        //     }
        // }
        Ok(())
    }
}

impl Analyzer for EnumAnalyzer {
    type Error = Infallible;
    type Node = EnumDeclaration;

    fn analyze<'c, C: Context<'c>>(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut C,
    ) -> TraversingResult<Self::Error> {
        let node = guard.allow_only(VisitSide::Entering)?;
        let tree = context.tree();

        if node.contents(&tree, node.token()).is_empty() {
            let rlt: Option<&Enum> = context.access(&*node);
            match rlt {
                None => {
                    warn_about_broken_rlt::<Enum>();
                    context.add_report(vec![], EmptyBody);
                }
                Some(
                    Enum::Stack {
                        contents: Some(contents),
                        ..
                    }
                    | Enum::Heap {
                        contents: Some(contents),
                        ..
                    },
                ) => context.add_report(
                    vec![contents.left.location(), contents.right.location()],
                    EmptyBody,
                ),
                _ => {}
            }
        }
        Ok(())
    }
}
