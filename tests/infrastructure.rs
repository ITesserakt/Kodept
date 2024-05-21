use std::borrow::Cow;
use std::fmt::Display;
use std::path::Path;

use nom_supreme::final_parser::final_parser;
use rstest::rstest;

use kodept::loader::Loader;
use kodept::macro_context::DefaultContext;
use kodept::parse_error::Reportable;
use kodept::read_code_source::ReadCodeSource;
use kodept::stage::PredefinedTraverseSet;
use kodept::top_parser;
use kodept_ast::ast_builder::ASTBuilder;
use kodept_ast::traits::{Accessor, Identifiable, Linker};
use kodept_core::code_point::CodePoint;
use kodept_core::code_source::CodeSource;
use kodept_core::ConvertibleToRef;
use kodept_core::structure::rlt::RLT;
use kodept_core::structure::span::CodeHolder;
use kodept_macros::erased::ErasedMacro;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::traits::{FileContextual, MutableContext, Reporter};
use kodept_parse::ParseError;
use kodept_parse::token_stream::TokenStream;
use kodept_parse::tokenizer::Tokenizer;

pub const EXAMPLES_FOLDER: &str = "examples";

struct CodeProvider<'a>(&'a str);

impl CodeHolder for CodeProvider<'_> {
    fn get_chunk(&self, at: CodePoint) -> Cow<str> {
        Cow::Borrowed(&self.0[at.offset..at.offset + at.length])
    }
}

fn get_code_source(name: impl Display) -> CodeSource {
    let loader = Loader::file()
        .with_starting_path(Path::new(&format!("{EXAMPLES_FOLDER}/{name}.kd")))
        .build()
        .expect("Cannot get given path");

    loader
        .into_sources()
        .pop()
        .expect("No source found with provided name")
}

fn get_rlt(source: &ReadCodeSource) -> RLT {
    let tokens = Tokenizer::new(source.contents()).into_vec();
    let tokens = TokenStream::new(&tokens);
    let result = final_parser(top_parser)(tokens)
        .map_err(|it: ParseError| it.to_diagnostics())
        .expect("Cannot parse");
    result
}

#[rstest]
#[case("church")]
#[case("test")]
#[case("fibonacci")]
#[case("rule110")]
fn test_typing(#[case] name: &str) {
    let source = ReadCodeSource::try_from(get_code_source(name)).expect("Cannot read source");
    let provider = CodeProvider(source.contents());
    let rlt = get_rlt(&source);

    let (ast, rlt_accessor) = ASTBuilder.recursive_build(&rlt.0, &provider);
    let ast = ast.build();

    let context = DefaultContext::new(source.with_filename(|_| ReportCollector::new()), rlt_accessor, ast);
    
    PredefinedTraverseSet::default().into_inner().traverse(context)
        .map_err(|it| it.0)
        .expect("Success");
}
