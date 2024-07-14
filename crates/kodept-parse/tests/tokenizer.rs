use kodept_parse::token_match::TokenMatch;
use lazy_static::lazy_static;
use kodept_parse::tokenizer::LazyTokenizer;

const FILENAME: &str = "tests/testing_file.kd";

lazy_static! {
    static ref FILE_CONTENTS: &'static str = std::fs::read_to_string(FILENAME).unwrap().leak();
    static ref TOKENS: &'static [TokenMatch<'static>] = {
        let tokenizer = LazyTokenizer::new(*FILE_CONTENTS);
        tokenizer.into_vec().leak()
    };
}

mod default {
    use kodept_parse::tokenizer::LazyTokenizer;
    use crate::{FILE_CONTENTS, TOKENS};

    #[test]
    fn test_impl() {
        let tokenizer = LazyTokenizer::new(*FILE_CONTENTS);
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(tokens, *TOKENS);
    }
}

#[cfg(feature = "pest")]
mod pest {
    use kodept_parse::grammar::PestKodeptParser;
    use crate::{FILE_CONTENTS, TOKENS};
    use kodept_parse::lexer::Token;

    #[test]
    fn test_impl() {
        let tokenizer = PestKodeptParser::new(*FILE_CONTENTS);
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(
            tokens,
            TOKENS
                .iter()
                .filter(|it| !matches!(it.token, Token::Ignore(_)))
                .cloned()
                .collect::<Vec<_>>()
        );
    }
}

#[cfg(feature = "peg")]
mod peg {
    use crate::{FILE_CONTENTS, TOKENS};
    use kodept_parse::grammar::KodeptParser as Tokenizer;

    #[test]
    fn test_impl() {
        let tokenizer = Tokenizer::new(*FILE_CONTENTS);
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(tokens, *TOKENS);
    }
}
