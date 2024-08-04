
use kodept_parse::token_match::TokenMatch;
use kodept_parse::tokenizer::LazyTokenizer;

const FILENAME: &str = "tests/testing_file.kd";

fn get_file_contents() -> &'static str {
    std::fs::read_to_string(FILENAME).unwrap().leak()
}

fn get_tokens() -> &'static [TokenMatch<'static>] {
    let file_contents: &'static str = std::fs::read_to_string(FILENAME).unwrap().leak();
    let tokens: &'static [TokenMatch<'static>] = {
        let tokenizer = LazyTokenizer::new(file_contents);
        tokenizer.into_vec().leak()
    };
    tokens
}

mod default {
    use kodept_parse::lexer::NomLexer;
    use kodept_parse::tokenizer::GenericLazyTokenizer;
    use crate::{get_file_contents, get_tokens};

    #[test]
    fn test_impl() {
        let tokenizer =
            GenericLazyTokenizer::new(get_file_contents(), NomLexer::new());
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(tokens, get_tokens());
    }
}

#[cfg(feature = "pest")]
mod pest {
    use kodept_parse::lexer::PestLexer;
    use kodept_parse::tokenizer::{EagerTokenizer};
    use crate::{get_file_contents, get_tokens};

    #[test]
    fn test_impl() {
        let tokenizer = EagerTokenizer::new(get_file_contents(), PestLexer::new());
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(tokens, get_tokens());
    }
}

#[cfg(feature = "peg")]
mod peg {
    use kodept_parse::lexer::PegLexer;
    use kodept_parse::tokenizer::{EagerTokenizer};
    use crate::{get_file_contents, get_tokens};

    #[test]
    fn test_impl() {
        let tokenizer = EagerTokenizer::new(get_file_contents(), PegLexer::<false>::new());
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(tokens, get_tokens());
    }
}
