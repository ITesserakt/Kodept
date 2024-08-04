use extend::ext;

pub mod lexer;
pub mod parser;

pub mod token_match;
pub mod token_stream;
pub mod tokenizer;

pub mod error;

#[cfg(feature = "peg")]
mod peg;
#[cfg(feature = "pest")]
mod pest;
#[cfg(feature = "nom")]
mod nom;

pub mod common;

#[ext]
impl<T> Option<T> {
    #[inline]
    fn map_into<U: From<T>>(self) -> Option<U> {
        self.map(|x| x.into())
    }
}

