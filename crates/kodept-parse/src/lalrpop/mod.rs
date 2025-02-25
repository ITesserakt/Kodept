mod error;
mod lexer;

mod lexer_implementation {
    #[allow(unused_imports)]
    #[allow(unreachable_pub)]
    #[rustfmt::skip]
    include!("grammar/lexer.rs");
}

pub use lexer::Lexer;
