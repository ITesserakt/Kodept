use std::path::PathBuf;
use std::string::FromUtf8Error;

use clap::{Parser, ValueEnum};
use derive_more::Display;
use thiserror::Error;

use crate::cli::traits::Command;
use kodept::source_files::SourceView;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::ErrorReported;

#[derive(Debug, ValueEnum, Clone, Display)]
enum InspectingOptions {
    Tokenizer,
    Parser,
    Both,
}

#[derive(Debug, Parser, Clone)]
pub struct InspectParser {
    /// Controls which step will be traced
    #[arg(default_value = "both", long = "inspect")]
    option: InspectingOptions,
    /// Additionally launch `pegviz` to produce html output
    #[arg(default_value_t = true, short = 'p', long = "pegviz")]
    use_pegviz: bool,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
enum LaunchPegvizError {
    #[error(transparent)]
    OutputIsNotUTF8(#[from] FromUtf8Error),
    #[error("`pegviz` exited with non-zero exit code: {0}")]
    NonZeroExit(i32),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

#[cfg(feature = "trace")]
#[derive(Debug, Error)]
enum InspectError<A> {
    #[error("Error happened while parsing")]
    TokenizationError(kodept_parse::error::ParseErrors<A>),
    #[error(transparent)]
    RedirectError(#[from] gag::RedirectError<std::fs::File>),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Pegviz(#[from] LaunchPegvizError),
}

#[cfg(not(feature = "trace"))]
impl Command for InspectParser {
    type Params = PathBuf;

    fn exec_for_source(
        &self,
        source: SourceView,
        collector: &mut ReportCollector,
        _: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        #[derive(Error, Debug)]
        #[error("Program is compiled without inspecting support")]
        struct Unsupported;

        collector.report(*source.id, Unsupported);
        Err(ErrorReported::new())
    }
}

#[cfg(feature = "trace")]
impl InspectParser {
    fn launch_pegviz<P: AsRef<std::path::Path>>(
        &self,
        input_file_path: P,
    ) -> Result<(), LaunchPegvizError> {
        use std::fs::File;
        use std::process::{Command, Output};
        use tracing::{debug, error, info, warn};

        if !self.use_pegviz {
            return Ok(());
        }

        let output_path = input_file_path.as_ref().with_extension("html");
        let input_file = File::open(input_file_path)?;
        let Output { status, stdout, .. } = Command::new("pegviz")
            .args(["--output".into(), output_path])
            .stdin(input_file)
            .output()?;
        let stdout = String::from_utf8(stdout)?;
        stdout.lines().for_each(|line| match line.split_once(":") {
            None if line.starts_with("= pegviz generated to") => {
                info!("{}", line.strip_prefix("= ").unwrap())
            }
            None => debug!("{line}"),
            Some((a, _)) if a.contains("error") => error!("{line}"),
            Some(_) => debug!("{line}"),
        });

        match status.code() {
            None => warn!("`pegviz` exited by signal"),
            Some(0) => {}
            Some(code) => Err(LaunchPegvizError::NonZeroExit(code))?,
        }
        Ok(())
    }

    fn inspect_tokenizer(
        &self,
        source: &SourceView,
        file_output_path: &std::path::Path,
    ) -> Result<(), InspectError<String>> {
        use kodept_parse::{
            lexer::PegLexer, tokenizer::EagerTokenizer, tokenizer::Tok, tokenizer::TokCtor,
        };
        use std::fs::File;
        use InspectError::TokenizationError;

        let file = File::create(file_output_path.with_extension("tok.peg"))?;
        {
            let _gag = gag::Redirect::stdout(file)?;
            EagerTokenizer::new(source.contents(), PegLexer::<true>::new())
                .try_collect_adapted::<String>()
                .map_err(TokenizationError)?;
        }

        self.launch_pegviz(file_output_path.with_extension("tok.peg"))?;
        Ok(())
    }

    fn inspect_parser(
        &self,
        source: &SourceView,
        file_output_path: &std::path::Path,
    ) -> Result<(), InspectError<String>> {
        use kodept_parse::{
            lexer::PegLexer,
            parser::{parse_from_top, PegParser},
            token_stream::TokenStream,
            tokenizer::{EagerTokenizer, Tok, TokCtor},
        };
        use std::fs::File;
        use InspectError::TokenizationError;

        let tokens = EagerTokenizer::new(source.contents(), PegLexer::<false>::new())
            .try_collect_adapted::<String>()
            .map_err(TokenizationError)?;
        let tokens = TokenStream::new(&tokens);

        let file = File::create(file_output_path.with_extension("par.peg"))?;
        {
            let _gag = gag::Redirect::stdout(file)?;
            let _ = parse_from_top(tokens, PegParser::<true>::new());
        }

        self.launch_pegviz(file_output_path.with_extension("par.peg"))?;
        Ok(())
    }
}

#[cfg(feature = "trace")]
impl Command for InspectParser {
    type Params = PathBuf;

    fn exec_for_source(
        &self,
        source: SourceView,
        collector: &mut ReportCollector,
        output_path: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        let filename = source.path();
        let source_name = filename.build_file_path();
        let file_output_path = output_path.join(source_name);
        match match self.option {
            InspectingOptions::Tokenizer => self.inspect_tokenizer(&source, &file_output_path),
            InspectingOptions::Parser => self.inspect_parser(&source, &file_output_path),
            InspectingOptions::Both => {
                if let Err(e) = self.inspect_tokenizer(&source, &file_output_path) {
                    collector.report(*source.id, e);
                }
                self.inspect_parser(&source, &file_output_path)
            }
        } {
            Ok(_) => Ok(()),
            Err(e) => {
                collector.report(*source.id, e);
                Err(ErrorReported::new())
            }
        }
    }
}
