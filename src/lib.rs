use kodept_core::structure::rlt::RLT;
use kodept_parse::parser::file;
use kodept_parse::token_stream::TokenStream;
use kodept_parse::ParseResult;

pub mod codespan_settings;
pub mod loader;
pub mod macro_context;
pub mod parse_error;
pub mod read_code_source;
pub mod traversing;
pub mod utils;
pub mod stage;

pub fn top_parser(input: TokenStream) -> ParseResult<RLT> {
    match file::grammar(input) {
        Ok(x) => Ok((x.0, RLT(x.1))),
        Err(e) => Err(e),
    }
}
