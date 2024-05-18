#![feature(iterator_try_collect)]

use std::borrow::Cow;
use std::fmt::Display;
use std::path::Path;
use std::rc::{Rc, Weak};

use nom_supreme::final_parser::final_parser;
use rstest::rstest;

use kodept::loader::Loader;
use kodept::parse_error::Reportable;
use kodept::read_code_source::ReadCodeSource;
use kodept::top_parser;
use kodept_ast::ast_builder::ASTBuilder;
use kodept_ast::graph::{GenericASTNode, PermTkn, SyntaxTree};
use kodept_ast::rlt_accessor::{RLTAccessor, RLTFamily};
use kodept_ast::traits::{Accessor, Identifiable, Linker};
use kodept_core::code_point::CodePoint;
use kodept_core::code_source::CodeSource;
use kodept_core::ConvertibleToRef;
use kodept_core::file_relative::CodePath;
use kodept_core::structure::rlt::RLT;
use kodept_core::structure::span::CodeHolder;
use kodept_interpret::operator_desugaring::BinaryOperatorExpander;
use kodept_interpret::semantic_analyzer::ScopeAnalyzer;
use kodept_interpret::type_checker::TypeChecker;
use kodept_macros::erased::ErasedMacro;
use kodept_macros::error::report::Report;
use kodept_macros::Macro;
use kodept_macros::traits::{FileContextual, Reporter};
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

struct FakeContext(Rc<SyntaxTree>, RLTAccessor);

impl Linker for FakeContext {
    fn link<A, B>(&mut self, _: &A, _: &B)
    where
        A: Identifiable + Into<GenericASTNode>,
        B: Into<RLTFamily> + Clone,
    {
        todo!()
    }

    fn link_existing<A, B>(&mut self, _: A, _: &B) -> A
    where
        A: Identifiable + Into<GenericASTNode>,
        B: Identifiable + Into<GenericASTNode>,
    {
        todo!()
    }
}

impl Accessor for FakeContext {
    fn access<A, B>(&self, _: &A) -> Option<&B>
    where
        A: Identifiable + Into<GenericASTNode>,
        RLTFamily: ConvertibleToRef<B>,
    {
        todo!()
    }

    fn access_unknown<A>(&self, a: &A) -> Option<RLTFamily>
    where
        A: Identifiable + Into<GenericASTNode>,
    {
        self.1.access_unknown(a).cloned()
    }

    fn tree(&self) -> Weak<SyntaxTree> {
        Rc::downgrade(&self.0)
    }
}

impl Reporter for FakeContext {
    fn report(&mut self, _: Report) {
        todo!()
    }
}

impl FileContextual for FakeContext {
    fn file_path(&self) -> CodePath {
        CodePath::ToMemory("Test environment".to_string())
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
    let ast = Rc::new(ast.build());
    let mut token = PermTkn::new();

    let mut scope = ScopeAnalyzer::new();
    let mut context = FakeContext(ast.clone(), rlt_accessor);
    
    ast.dfs().for_each(|(node, side)| {
        scope.transform(node, side, &mut token, &mut context)
            .into_result()
            .expect("Success");
    });
    
    let scope = Rc::new(scope.into_inner());
    let mut type_checker = TypeChecker::new(scope.clone());
    
    ast.dfs().for_each(|(node, side)| {
        type_checker.transform(node, side, &mut token, &mut context)
            .into_result()
            .expect("Success");
    });
    
    dbg!(scope);
}
