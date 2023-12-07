pub(crate) mod block_level;
pub(crate) mod code_flow;
pub(crate) mod expression;
pub mod file;
pub(crate) mod function;
pub(crate) mod literal;
pub(crate) mod nom;
pub(crate) mod operator;
pub(crate) mod parameter;
pub(crate) mod term;
pub(crate) mod top_level;
pub(crate) mod r#type;

pub mod error {
    pub use crate::parser::nom::TokenVerificationError;
}

pub(crate) mod macros {
    #[macro_export]
    macro_rules! function {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            name.strip_suffix("::f").unwrap()
        }};
    }
}
