use crate::analyzer::Analyzer;
use crate::error::report::ReportMessage;
use crate::traits::Context;
use codespan_reporting::diagnostic::Severity;
use itertools::Itertools;
use kodept_ast::visitor::visit_side::{VisitGuard, VisitSide};
use kodept_ast::visitor::TraversingResult;
use kodept_ast::{BlockLevel, ExpressionBlock};
use kodept_core::impl_named;
use kodept_core::structure::rlt::Variable;
use kodept_core::structure::{rlt, Located};
use std::convert::Infallible;

#[derive(Debug)]
pub struct VariableUniquenessAnalyzer;

impl_named!(VariableUniquenessAnalyzer);

pub struct DuplicatedVariable(String);

impl From<DuplicatedVariable> for ReportMessage {
    fn from(value: DuplicatedVariable) -> Self {
        Self::new(
            Severity::Error,
            "SE002",
            format!("Variable `{}` has duplicates in one block", value.0),
        )
    }
}

impl Analyzer for VariableUniquenessAnalyzer {
    type Error = Infallible;
    type Node<'n> = &'n ExpressionBlock;

    fn analyze<'n, 'c, C: Context<'c>>(
        &self,
        guard: VisitGuard<Self::Node<'n>>,
        context: &mut C,
    ) -> TraversingResult<Self::Error> {
        let node = guard.allow_only(VisitSide::Exiting)?;
        let variables = node
            .items
            .iter()
            .filter_map(|it| match it {
                BlockLevel::InitVar(x) => Some(x),
                _ => None,
            })
            .group_by(|it| &it.variable.name);

        for (name, variables) in variables.into_iter() {
            let variables = variables.collect_vec();
            if variables.len() > 1 {
                context.add_report(
                    variables
                        .into_iter()
                        .filter_map(|it| context.access(it))
                        .map(|it: &rlt::InitializedVariable| match &it.variable {
                            Variable::Immutable { id, .. } | Variable::Mutable { id, .. } => {
                                id.location()
                            }
                        })
                        .collect(),
                    DuplicatedVariable(name.clone()),
                )
            }
        }

        Ok(())
    }
}