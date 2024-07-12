#[cfg(feature = "nom")]
pub mod file;
#[cfg(feature = "nom")]
pub(crate) mod block_level;
#[cfg(feature = "nom")]
pub(crate) mod code_flow;
#[cfg(feature = "nom")]
pub(crate) mod expression;
#[cfg(feature = "nom")]
pub(crate) mod function;
#[cfg(feature = "nom")]
pub(crate) mod literal;
#[cfg(feature = "nom")]
pub(crate) mod nom;
#[cfg(feature = "nom")]
pub(crate) mod operator;
#[cfg(feature = "nom")]
pub(crate) mod parameter;
#[cfg(feature = "nom")]
pub(crate) mod term;
#[cfg(feature = "nom")]
pub(crate) mod top_level;
#[cfg(feature = "nom")]
pub(crate) mod r#type;

pub(crate) mod common;

pub mod error {
    #[cfg(feature = "nom")]
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
