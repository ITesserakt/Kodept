
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
    use kodept_parse::tokenizer::GenericLazyTokenizer;
    use crate::{get_file_contents, get_tokens};

    #[test]
    fn test_impl() {
        let tokenizer =
            GenericLazyTokenizer::new(get_file_contents(), kodept_parse::lexer::nom_parse_token);
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(tokens, get_tokens());
    }
}

#[cfg(feature = "pest")]
mod pest {
    use kodept_parse::grammar::PestKodeptParser;
    use crate::{get_file_contents, get_tokens};

    #[test]
    fn test_impl() {
        let tokenizer = PestKodeptParser::new(get_file_contents());
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(tokens, get_tokens());
    }
}

#[cfg(feature = "peg")]
mod peg {
    use kodept_parse::grammar::KodeptParser as Tokenizer;
    use crate::{get_file_contents, get_tokens};

    #[test]
    fn test_impl() {
        let tokenizer = Tokenizer::new(get_file_contents());
        let tokens = tokenizer.into_vec();
        similar_asserts::assert_eq!(tokens, get_tokens());
    }
}
