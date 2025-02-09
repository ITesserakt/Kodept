// mod convert_model;
// mod node_family;
pub mod operator_desugaring;
mod scope;
// pub mod semantic_analyzer;
mod symbol;
// pub mod type_checker;
pub mod scope_analyzer;
pub mod reference_resolver;
pub mod linting;
pub mod macros;
pub mod dot_formatter;

pub mod path {
    use std::fmt::{Display, Formatter};
    use kodept_ast::interning::SharedStr;
    use kodept_ast::ReferenceContext;

    #[derive(Debug, Clone)]
    pub struct Path {
        pub context: ReferenceContext,
        pub ident: SharedStr
    }

    impl Display for Path {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.context.global {
                write!(f, "::")?;
            }
            for item in &self.context.items {
                write!(f, "{item}::")?;
            }
            write!(f, "{}", self.ident)?;
            
            Ok(())
        }
    }
}

pub(crate) type Path = String;
