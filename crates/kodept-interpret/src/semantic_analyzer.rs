use std::ops::Deref;

use extend::ext;
use tracing::debug;

use kodept_ast::graph::{GhostToken, NodeUnion, SyntaxTree};
use kodept_ast::rlt_accessor::RLTFamily;
use kodept_ast::traits::IntoASTFamily;
use kodept_ast::visitor::visit_side::{VisitGuard, VisitSide};
use kodept_ast::visitor::TraversingResult;
use kodept_ast::{
    BodiedFunctionDeclaration, EnumDeclaration, Identifier, InitializedVariable, ModuleDeclaration,
    Parameter, Reference,
};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::{rlt, Located};
use kodept_core::{ConvertibleTo, Named};
use kodept_macros::analyzer::Analyzer;
use kodept_macros::error::report::ReportMessage;
use kodept_macros::traits::{Context, UnrecoverableError};
use kodept_macros::warn_about_broken_rlt;

use crate::scope::ScopedSymbolTable;
use crate::semantic_analyzer::wrapper::AnalyzingNode;
use crate::symbol::{TypeSymbol, VarSymbol};
use crate::Errors;

mod wrapper {
    use derive_more::{From, Into};

    use kodept_ast::graph::{GenericASTNode, NodeUnion};
    use kodept_ast::{
        wrapper, BodiedFunctionDeclaration, EnumDeclaration, InitializedVariable,
        ModuleDeclaration, Parameter, Reference,
    };

    wrapper! {
        #[derive(From, Into)]
        #[into(ref)]
        wrapper AnalyzingNode {
            module(ModuleDeclaration) = GenericASTNode::Module(x) => Some(x),
            var_decl(InitializedVariable) = GenericASTNode::InitializedVariable(x) => Some(x),
            enum(EnumDeclaration) = GenericASTNode::Enum(x) => Some(x),
            reference(Reference) = GenericASTNode::Reference(x) => Some(x),
            function(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction(x) => Some(x)
            parameter(Parameter) = n if Parameter::contains(n) => n.try_into().ok()
        }
    }
}

pub struct SemanticAnalyzer {
    current_scope: ScopedSymbolTable,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self {
            current_scope: ScopedSymbolTable::new("::", None),
        }
    }
}

#[ext]
impl<E: Into<ReportMessage>> Result<(), E> {
    fn report_errors<'c, F, U: 'c>(
        self,
        at: &impl IntoASTFamily,
        context: &mut impl Context<'c>,
        func: F,
    ) where
        RLTFamily<'c>: ConvertibleTo<&'c U>,
        F: Fn(&U) -> Vec<CodePoint>,
    {
        if let Err(error) = self {
            let points = context.access(at).map_or_else(
                || {
                    warn_about_broken_rlt::<U>();
                    vec![]
                },
                func,
            );
            context.add_report(points, error);
        }
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Named for SemanticAnalyzer {}

impl Analyzer for SemanticAnalyzer {
    type Error = UnrecoverableError;
    type Node = AnalyzingNode;

    fn analyze<'c, C: Context<'c>>(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut C,
    ) -> TraversingResult<Self::Error> {
        let (node, side) = guard.allow_all();
        let tree = context.tree();
        if let Some(module) = node.as_module() {
            self.visit_module(module, side)
                .report_errors(module, context, |rlt: &rlt::Module| vec![rlt.location()]);
        } else if let Some(var) = node.as_var_decl() {
            self.visit_var_decl(var, side, node.token(), &tree)
                .report_errors(var, context, |rlt: &rlt::InitializedVariable| {
                    vec![rlt.variable.location()]
                });
        } else if let Some(enumeration) = node.as_enum() {
            self.visit_enum(enumeration, side, node.token(), &tree)
                .report_errors(enumeration, context, |rlt: &rlt::Enum| {
                    vec![rlt.id().location()]
                });
        } else if let Some(var) = node.as_reference() {
            self.visit_reference(var)
                .report_errors(var, context, |rlt: &rlt::Reference| vec![rlt.location()]);
        } else if let Some(func) = node.as_function() {
            self.visit_func(func, side).report_errors(
                func,
                context,
                |rlt: &rlt::BodiedFunction| vec![rlt.id.location()],
            );
        } else if let Some(param) = node.as_parameter() {
            let result = self.visit_parameter(param, side, &tree, node.token());
            if let Some(typed) = param.as_typed() {
                result.report_errors(typed, context, |rlt: &rlt::TypedParameter| {
                    vec![rlt.location()]
                });
            } else if let Some(untyped) = param.as_untyped() {
                result.report_errors(untyped, context, |rlt: &rlt::UntypedParameter| {
                    vec![rlt.id.location()]
                })
            }
        }

        if side == VisitSide::Exiting && AnalyzingNode::contains(node.deref().into()) {
            debug!("{:#?}", self.current_scope);
        }
        Ok(())
    }
}

impl SemanticAnalyzer {
    fn visit_module(&mut self, module: &ModuleDeclaration, side: VisitSide) -> Result<(), Errors> {
        if side == VisitSide::Entering {
            self.current_scope.new_layer(module.name.clone());
        } else if side == VisitSide::Exiting {
            self.current_scope
                .replace_with_enclosing_scope(&module.name)?;
        }
        Ok(())
    }

    fn visit_var_decl(
        &mut self,
        decl: &InitializedVariable,
        side: VisitSide,
        token: &GhostToken,
        tree: &SyntaxTree,
    ) -> Result<(), Errors> {
        if side != VisitSide::Entering {
            return Ok(());
        }
        let ty = decl.variable(tree, token).assigned_type(tree, token);
        let ty_symbol = match ty.and_then(|it| it.as_type_name()) {
            None => panic!("Complex types is not supported yet"),
            Some(x) => Some(self.current_scope.lookup_type(&x.name, false)?),
        };
        self.current_scope.insert(VarSymbol::new(
            decl.variable(tree, token).name.clone(),
            ty_symbol,
        ))?;
        Ok(())
    }

    fn visit_enum(
        &mut self,
        decl: &EnumDeclaration,
        side: VisitSide,
        token: &GhostToken,
        tree: &SyntaxTree,
    ) -> Result<(), Errors> {
        if side != VisitSide::Entering {
            return Ok(());
        }
        let ty = self
            .current_scope
            .insert(TypeSymbol::user(decl.name.clone()))?;
        for variant in decl.contents(tree, token) {
            self.current_scope
                .insert(VarSymbol::new(variant.name.clone(), Some(ty.clone())))?;
        }
        Ok(())
    }

    fn visit_reference(&mut self, var: &Reference) -> Result<(), Errors> {
        match &var.ident {
            Identifier::TypeReference { name } => {
                self.current_scope.lookup_type(name, false)?;
            }
            Identifier::Reference { name } => {
                self.current_scope.lookup_var(name, false)?;
            }
        }
        Ok(())
    }

    fn visit_func(
        &mut self,
        func: &BodiedFunctionDeclaration,
        side: VisitSide,
    ) -> Result<(), Errors> {
        if side == VisitSide::Entering {
            self.current_scope
                .insert(VarSymbol::new(func.name.clone(), None))?;
            self.current_scope.new_layer(func.name.clone());
        } else if side == VisitSide::Exiting {
            self.current_scope
                .replace_with_enclosing_scope(&func.name)?;
        }
        Ok(())
    }

    fn visit_parameter(
        &mut self,
        param: &Parameter,
        side: VisitSide,
        tree: &SyntaxTree,
        token: &GhostToken,
    ) -> Result<(), Errors> {
        if side != VisitSide::Entering {
            return Ok(());
        }
        if let Some(typed) = param.as_typed() {
            let ty = match typed.parameter_type(tree, token).as_type_name() {
                None => panic!("Complex types is not supported yet"),
                Some(x) => Some(self.current_scope.lookup_type(&x.name, false)?),
            };
            self.current_scope
                .insert(VarSymbol::new(typed.name.clone(), ty))?;
        } else if let Some(untyped) = param.as_untyped() {
            self.current_scope
                .insert(VarSymbol::new(untyped.name.clone(), None))?;
        }
        Ok(())
    }
}
