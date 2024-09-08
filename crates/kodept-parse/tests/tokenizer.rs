use kodept_parse::lexer::PegLexer;
use kodept_parse::token_match::TokenMatch;
use kodept_parse::tokenizer::{LazyTokenizer, Tok, TokCtor};
use std::fmt::Debug;

const FILENAME: &str = "tests/testing_file.kd";

fn get_file_contents() -> &'static str {
    std::fs::read_to_string(FILENAME).unwrap().leak()
}

fn get_tokens() -> &'static [TokenMatch<'static>] {
    let file_contents: &'static str = std::fs::read_to_string(FILENAME).unwrap().leak();
    let tokens: &'static [TokenMatch<'static>] = {
        let tokenizer = LazyTokenizer::new(file_contents, PegLexer::<true>::new());
        tokenizer.into_vec().leak()
    };
    tokens
}

#[inline(always)]
fn make_test_impl<'t, T, F>(lexer: F)
where
    T::Error: Debug,
    T: Tok<'t> + TokCtor<'t, F>,
{
    let tokenizer = T::new(get_file_contents(), lexer);
    let tokens = tokenizer.into_vec();
    similar_asserts::assert_eq!(tokens, get_tokens());
}

mod nom {
    use crate::make_test_impl;
    use kodept_parse::lexer::NomLexer;
    use kodept_parse::tokenizer::LazyTokenizer;

    #[test]
    fn test_lazy() {
        make_test_impl::<LazyTokenizer<_>, _>(NomLexer::new());
    }
    // 
    // #[test]
    // fn test_parallel() {
    //     make_test_impl::<ParallelTokenizer<_>, _>(NomLexer::new())
    // }
}

mod peg {
    use crate::make_test_impl;
    use kodept_parse::lexer::PegLexer;
    use kodept_parse::tokenizer::{EagerTokenizer, LazyTokenizer, ParallelTokenizer};

    #[test]
    fn test_lazy() {
        make_test_impl::<LazyTokenizer<_>, _>(PegLexer::<true>::new());
    }
    
    #[test]
    fn test_eager() {
        make_test_impl::<EagerTokenizer<_, _>, _>(PegLexer::<true>::new());
    }
    
    #[test]
    fn test_parallel() {
        make_test_impl::<ParallelTokenizer<_>, _>(PegLexer::<true>::new());
    }
}

mod pest {
    use crate::make_test_impl;
    use kodept_parse::lexer::PestLexer;
    use kodept_parse::tokenizer::{EagerTokenizer, LazyTokenizer, ParallelTokenizer};

    #[test]
    fn test_lazy() {
        make_test_impl::<LazyTokenizer<_>, _>(PestLexer::new());
    }
    
    #[test]
    fn test_eager() {
        make_test_impl::<EagerTokenizer<_, _>, _>(PestLexer::new());
    }
    
    #[test]
    fn test_parallel() {
        make_test_impl::<ParallelTokenizer<_>, _>(PestLexer::new())
    }
}
