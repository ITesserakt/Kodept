mod compatibility;
mod parser;
mod error;

mod kodept {
    #[allow(unused_imports)]
    #[allow(unreachable_pub)]
    #[rustfmt::skip]
    include!("grammar/kodept.rs");
}

pub use parser::Parser;

#[cfg(test)]
mod tests {
    use super::kodept;

    #[test]
    fn test_grammar_compiled() {
        let result = kodept::TermParser::new().parse("((22))");
        assert!(matches!(result, Ok(22)))
    }
}
