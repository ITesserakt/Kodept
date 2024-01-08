use std::convert::Infallible;

use codespan_reporting::diagnostic::Severity;
use itertools::Itertools;

use kodept_ast::visitor::visit_side::{VisitGuard, VisitSide};
use kodept_ast::visitor::TraversingResult;
use kodept_ast::{FileDeclaration, ModuleDeclaration, ModuleKind};
use kodept_core::impl_named;
use kodept_core::structure::rlt::Module;
use kodept_core::structure::Located;

use crate::analyzer::Analyzer;
use crate::error::report::ReportMessage;
use crate::traits::Context;
use crate::warn_about_broken_rlt;

#[derive(Debug)]
pub struct ModuleUniquenessAnalyzer;
#[derive(Debug)]
pub struct GlobalModuleAnalyzer;

impl_named!(ModuleUniquenessAnalyzer);
impl_named!(GlobalModuleAnalyzer);

pub struct DuplicatedModules(String);
pub struct NonGlobalModule(String);

impl From<DuplicatedModules> for ReportMessage {
    fn from(value: DuplicatedModules) -> Self {
        ReportMessage::new(
            Severity::Error,
            "SE001",
            format!("File contains duplicated module `{}`", value.0),
        )
    }
}

impl From<NonGlobalModule> for ReportMessage {
    fn from(value: NonGlobalModule) -> Self {
        ReportMessage::new(
            Severity::Warning,
            "SE002",
            format!(
                "Consider replace brackets in module statement `{}` to `=>` operator",
                value.0
            ),
        )
    }
}

impl Analyzer for GlobalModuleAnalyzer {
    type Error = Infallible;
    type Node = FileDeclaration;

    fn analyze<'n, 'c, C: Context<'c>>(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut C,
    ) -> TraversingResult<Self::Error> {
        let node = guard.allow_only(VisitSide::Entering)?;

        // if let [m @ ModuleDeclaration {
        //     kind: ModuleKind::Ordinary,
        //     name,
        //     ..
        // }] = node.modules(&context.tree(), &*token).as_slice()
        // {
        //     match context.access(*m) {
        //         Some(Module::Global { .. }) => {}
        //         Some(Module::Ordinary { lbrace, rbrace, .. }) => context.add_report(
        //             vec![lbrace.location(), rbrace.location()],
        //             NonGlobalModule(name.clone()),
        //         ),
        //         None => {
        //             warn_about_broken_rlt::<Module>();
        //             context.add_report(vec![], NonGlobalModule(name.clone()))
        //         }
        //     };
        //     Ok(())
        // } else {
        //     Ok(())
        // }
        Ok(())
    }
}

impl Analyzer for ModuleUniquenessAnalyzer {
    type Error = Infallible;
    type Node = FileDeclaration;

    fn analyze<'n, 'c, C: Context<'c>>(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut C,
    ) -> TraversingResult<Self::Error> {
        let tree = context.tree();
        let node = guard.allow_only(VisitSide::Entering)?;
        let group = node
            .modules(&tree, node.token())
            .into_iter()
            .group_by(|it| &it.name);
        let non_unique = group
            .into_iter()
            .map(|it| (it.0, it.1.collect_vec()))
            .filter(|(_, group)| group.len() > 1)
            .map(|(name, group)| {
                (
                    name.clone(),
                    group
                        .into_iter()
                        .filter_map(|it| context.access(it))
                        .map(|it: &Module| it.get_keyword().location())
                        .collect_vec(),
                )
            })
            .collect_vec();

        for (name, positions) in non_unique {
            context.add_report(positions, DuplicatedModules(name))
        }

        Ok(())
    }
}
