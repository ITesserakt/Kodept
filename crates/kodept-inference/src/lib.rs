//! This crate contains an algorithm to check (and infer) types based on
//! Hindley-Milner type system with constraints extension.
//! Also provides some structures, allowing to model those types.

pub mod algorithm_u;
pub mod assumption;
pub mod constraint;
// mod process;
pub mod engine;
pub mod process;
pub mod substitution;
pub mod traits;
pub mod r#type;

#[allow(unreachable_pub)]
mod utils {
    use std::fmt::{Display, Formatter};

    pub struct JoinedDisplay<'a, T>(T, &'a str, &'a str);

    impl<'a, T> JoinedDisplay<'a, T> {
        pub const fn enumerate(iter: T) -> Self {
            Self(iter, ", ", "")
        }

        pub const fn new(iter: T, separator: &'a str) -> Self {
            Self(iter, separator, "")
        }

        pub const fn with_prefix(mut self, prefix: &'a str) -> Self {
            self.2 = prefix;
            self
        }
    }

    impl<'a, 'b, T> Display for JoinedDisplay<'a, T>
    where
        T: IntoIterator<Item: Display> + Clone,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let mut first = true;
            for item in self.0.clone().into_iter() {
                if first {
                    first = false;
                    write!(f, "{}{item}", self.2)?;
                } else {
                    write!(f, "{}{}{item}", self.1, self.2)?;
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
