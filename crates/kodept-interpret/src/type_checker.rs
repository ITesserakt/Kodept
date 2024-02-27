use kodept_ast::graph::ChangeSet;
use kodept_ast::visitor::visit_side::{Skip, VisitGuard};
use kodept_ast::FunctionDeclaration;
use kodept_inference::Environment;
use kodept_macros::traits::{Context, UnrecoverableError};
use kodept_macros::Macro;

use crate::scope::SymbolTable;

pub struct TypeChecker {
    symbols: SymbolTable,
    env: Environment,
}

impl TypeChecker {
    fn populate_env(&mut self) {}

    pub fn new(symbol_table: SymbolTable) -> Self {
        let mut s: Vec<_> = symbol_table.into_symbols().into_iter().collect();
        s.sort_by_key(|it| it.0.len());
        dbg!(s);
        let mut this = Self {
            symbols: SymbolTable::new("".to_string()),
            env: Environment::default(),
        };
        this.populate_env();
        this
    }
}

impl Macro for TypeChecker {
    type Error = UnrecoverableError;
    type Node = FunctionDeclaration;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Result<ChangeSet, Skip<Self::Error>> {
        Ok(ChangeSet::empty())
    }
}
