//! This crate contains an algorithm to check (and infer) types based on
//! Hindley-Milner type system with constraints extension.
//! Also provides some structures, allowing to model those types.

pub mod algorithm_u;
// pub mod algorithm_w;
pub mod assumption;
pub mod constraint;
// mod process;
pub mod substitution;
pub mod traits;
pub mod r#type;

// pub mod prelude {
//     pub use super::process::{Continuation, DefaultExecutor, Infer, Suspend};
//     pub use super::traits::TypeInfer;
// }

#[allow(unreachable_pub)]
mod utils {
    use std::fmt::{Display, Formatter};

    pub struct JoinedDisplay<'a, T>(T, &'a str);

    impl<'a, T> JoinedDisplay<'static, T> {
        pub const fn enumerate(iter: T) -> Self {
            Self(iter, ", ")
        }

        pub fn join(self) -> String
        where
            T: Iterator<Item: Display>
        {
            use std::fmt::Write;

            let mut result = String::new();
            let mut first = true;
            for item in self.0 {
                if first {
                    first = false;
                    _ = write!(result, "{item}");
                } else {
                    _ = write!(result, "{}{item}", self.1)
                }
            }
            result
        }
    }

    impl<'a, 'b, T> Display for JoinedDisplay<'a, &'b T>
    where
        &'b T: IntoIterator<Item: Display>
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let mut first = true;
            for item in self.0.into_iter() {
                if first {
                    first = false;
                    write!(f, "{item}")?;
                } else {
                    write!(f, "{}{item}", self.1)?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
pub mod exported {
    pub use super::utils::JoinedDisplay;
}

pub const LOWER_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub const UPPER_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
