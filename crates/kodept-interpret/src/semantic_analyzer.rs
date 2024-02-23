use std::ops::Deref;

use tracing::debug;

use kodept_ast::{
    BodiedFunctionDeclaration, EnumDeclaration, Identifier, InitializedVariable, ModuleDeclaration,
    Parameter, Reference, StructDeclaration,
};
use kodept_ast::graph::{ChangeSet, GhostToken, NodeUnion, SyntaxTree};
use kodept_ast::visitor::visit_side::{VisitGuard, VisitSide};
use kodept_ast::visitor::visit_side::Skip;
use kodept_core::Named;
use kodept_core::structure::{Located, rlt};
use kodept_macros::error::report::ResultTEExt;
use kodept_macros::Macro;
use kodept_macros::traits::{Context, UnrecoverableError};

use crate::Errors;
use crate::Errors::TooComplex;
use crate::scope::SymbolTable;
use crate::semantic_analyzer::wrapper::AnalyzingNode;
use crate::symbol::{TypeSymbol, VarSymbol};

mod wrapper {
    use derive_more::{From, Into};

    use kodept_ast::{
        BodiedFunctionDeclaration, EnumDeclaration, InitializedVariable, ModuleDeclaration,
        Parameter, Reference, StructDeclaration, wrapper,
    };
    use kodept_ast::graph::{GenericASTNode, NodeUnion};

    wrapper! {
        #[derive(From, Into)]
        #[into(ref)]
        wrapper AnalyzingNode {
            module(ModuleDeclaration) = GenericASTNode::Module(x) => Some(x),
            var_decl(InitializedVariable) = GenericASTNode::InitializedVariable(x) => Some(x),
            enum(EnumDeclaration) = GenericASTNode::Enum(x) => Some(x),
            reference(Reference) = GenericASTNode::Reference(x) => Some(x),
            function(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction(x) => Some(x)
            parameter(Parameter) = n if Parameter::contains(n) => n.try_into().ok(),
            struct(StructDeclaration) = GenericASTNode::Struct(x) => Some(x),
        }
    }
}

pub struct SemanticAnalyzer {
    current_scope: SymbolTable,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self {
            current_scope: SymbolTable::new("".to_string()),
        }
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Named for SemanticAnalyzer {}

impl Macro for SemanticAnalyzer {
    type Error = UnrecoverableError;
    type Node = AnalyzingNode;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Result<ChangeSet, Skip<Self::Error>> {
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
                    vec![rlt.parameter_type.location()]
                });
            } else if let Some(untyped) = param.as_untyped() {
                result.report_errors(untyped, context, |rlt: &rlt::UntypedParameter| {
                    vec![rlt.id.location()]
                })
            }
        } else if let Some(structure) = node.as_struct() {
            self.visit_struct(structure, side).report_errors(
                structure,
                context,
                |rlt: &rlt::Struct| vec![rlt.id.location()],
            )
        }

        if side == VisitSide::Exiting && AnalyzingNode::contains(node.deref().into()) {
            debug!("{:#?}", self.current_scope);
        }
        Ok(ChangeSet::empty())
    }
}

impl SemanticAnalyzer {
    fn visit_module(&mut self, module: &ModuleDeclaration, side: VisitSide) -> Result<(), Errors> {
        if side == VisitSide::Entering {
            self.current_scope.begin_scope(module.name.clone())?;
        } else if side == VisitSide::Exiting {
            self.current_scope.end_scope(&module.name)?;
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
        let ty_symbol = match ty.map(|it| it.as_type_name()) {
            None => None,
            Some(None) => Err(TooComplex)?,
            Some(Some(x)) => Some(self.current_scope.lookup_type(&x.name, false)?),
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
        if side == VisitSide::Entering {
            let ty = self
                .current_scope
                .insert(TypeSymbol::user(decl.name.clone()))?;
            self.current_scope.begin_scope(decl.name.clone())?;
            for variant in decl.contents(tree, token) {
                self.current_scope
                    .insert(VarSymbol::new(variant.name.clone(), Some(ty.clone())))?;
            }
        }
        if side == VisitSide::Exiting {
            self.current_scope.end_scope(&decl.name)?;
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
            self.current_scope.begin_scope(func.name.clone())?;
        } else if side == VisitSide::Exiting {
            self.current_scope.end_scope(&func.name)?;
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
        if side == VisitSide::Exiting {
            return Ok(());
        }
        if let Some(typed) = param.as_typed() {
            let ty = match typed.parameter_type(tree, token).as_type_name() {
                None => Err(TooComplex)?,
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

    fn visit_struct(
        &mut self,
        structure: &StructDeclaration,
        side: VisitSide,
    ) -> Result<(), Errors> {
        if matches!(side, VisitSide::Entering | VisitSide::Leaf) {
            self.current_scope
                .insert(TypeSymbol::user(structure.name.clone()))?;
        }

        if side == VisitSide::Entering {
            self.current_scope.begin_scope(structure.name.clone())?;
        } else if side == VisitSide::Exiting {
            self.current_scope.end_scope(&structure.name)?;
        }
        Ok(())
    }
}
