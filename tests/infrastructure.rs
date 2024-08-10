use std::borrow::Cow;
use std::fmt::Display;
use std::num::NonZeroU16;
use std::path::Path;
use std::rc::Rc;

use insta::assert_debug_snapshot;

use kodept::loader::Loader;
use kodept::macro_context::DefaultContext;
use kodept::read_code_source::ReadCodeSource;
use kodept::steps::hlist::{HCons, HNil};
use kodept::steps::pipeline::Pipeline;
use kodept::steps::Step;
use kodept_ast::ast_builder::ASTBuilder;
use kodept_core::code_point::CodePoint;
use kodept_core::code_source::CodeSource;
use kodept_core::structure::rlt::RLT;
use kodept_core::structure::span::CodeHolder;
use kodept_inference::language::Language;
use kodept_interpret::operator_desugaring::*;
use kodept_interpret::semantic_analyzer::ScopeAnalyzer;
use kodept_interpret::type_checker::TypeChecker;
use kodept_interpret::{Cache, Witness};
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::traits::{MutableContext, UnrecoverableError};
use kodept_parse::parser::default_parse_from_top;
use kodept_parse::token_stream::TokenStream;
use kodept_parse::tokenizer::{LazyTokenizer, Tokenizer, TokenizerExt};

pub const EXAMPLES_FOLDER: &str = "examples";

struct CodeProvider<'a>(&'a str);

impl CodeHolder for CodeProvider<'_> {
    fn get_chunk(&self, at: CodePoint) -> Cow<str> {
        Cow::Borrowed(&self.0[at.as_range()])
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
    let tokens = LazyTokenizer::default(source.contents()).into_vec();
    let tokens = TokenStream::new(&tokens);
    let result = default_parse_from_top(tokens).expect("Cannot parse");
    result
}

fn common_steps(ctx: &mut impl MutableContext) -> Result<Cache<Rc<Language>>, UnrecoverableError> {
    Pipeline
        .define_step(HCons {
            head: AccessExpander::new(),
            tail: HCons {
                head: BinaryOperatorExpander::new(),
                tail: HCons {
                    head: UnaryOperatorExpander::new(),
                    tail: HNil,
                },
            },
        })
        .apply_with_context(ctx)?;

    let mut scopes = ScopeAnalyzer::new();
    Pipeline
        .define_step(HCons {
            head: &mut scopes,
            tail: HNil,
        })
        .apply_with_context(ctx)?;
    let scopes = scopes.into_inner();

    let mut type_checker = TypeChecker::new(
        &scopes,
        NonZeroU16::new(256).unwrap(),
        Witness::fact(
            AccessExpander,
            BinaryOperatorExpander,
            UnaryOperatorExpander,
        ),
    );
    Pipeline
        .define_step(HCons {
            head: &mut type_checker,
            tail: HNil,
        })
        .apply_with_context(ctx)?;
    Ok(type_checker.into_inner())
}

fn test_typing(name: &str) {
    let source = ReadCodeSource::try_from(get_code_source(name)).expect("Cannot read source");
    let provider = CodeProvider(source.contents());
    let rlt = get_rlt(&source);

    let (ast, rlt_accessor) = ASTBuilder.recursive_build(&rlt.0, &provider);
    let ast = ast.build();

    let mut context = DefaultContext::new(
        source.with_filename(|_| ReportCollector::new()),
        rlt_accessor,
        ast,
    );

    let contents = common_steps(&mut context).expect("Success");
    let mut values: Vec<(_, _)> = contents.into_iter().collect();
    values.sort_by_key(|it| it.0);

    assert_debug_snapshot!(values);
}

#[test]
#[ignore]
fn test_typing_on_church_encoding() {
    test_typing("church")
}

#[test]
#[ignore]
fn test_typing_on_test_file() {
    test_typing("test")
}

#[test]
#[ignore]
fn test_typing_on_rule110() {
    test_typing("rule110")
}

#[test]
#[ignore]
fn test_typing_on_fibonacci() {
    test_typing("fibonacci")
}
