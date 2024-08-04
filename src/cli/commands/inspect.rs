use std::path::PathBuf;
use std::string::FromUtf8Error;

use clap::{Parser, ValueEnum};
use derive_more::Display;
use thiserror::Error;

use kodept::codespan_settings::CodespanSettings;
use kodept::read_code_source::ReadCodeSource;
use kodept_macros::error::ErrorReported;
use kodept_macros::error::traits::ResultTEExt;

use crate::cli::traits::Command;

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
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        _: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        #[derive(Error, Debug)]
        #[error("Program is compiled without inspecting support")]
        struct Unsupported;

        let error = Err(Unsupported);
        error.or_emit(settings, &source, source.path())
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
        source: &ReadCodeSource,
        file_output_path: &std::path::Path,
    ) -> Result<(), InspectError<String>> {
        use kodept_parse::{lexer::PegLexer, tokenizer::EagerTokenizer};
        use std::fs::File;
        use InspectError::TokenizationError;

        let file = File::create(file_output_path.with_extension("tok.peg"))?;
        {
            let _gag = gag::Redirect::stdout(file)?;
            EagerTokenizer::try_new(source.contents(), PegLexer::<true>::new())
                .map_err(|it| TokenizationError(it))?;
        }

        self.launch_pegviz(file_output_path.with_extension("tok.peg"))?;
        Ok(())
    }

    fn inspect_parser(
        &self,
        source: &ReadCodeSource,
        file_output_path: &std::path::Path,
    ) -> Result<(), InspectError<String>> {
        use kodept_parse::{
            lexer::PegLexer,
            parser::{parse_from_top, PegParser},
            token_stream::TokenStream,
            tokenizer::EagerTokenizer,
        };
        use std::fs::File;
        use InspectError::TokenizationError;

        let tokenizer = EagerTokenizer::try_new(source.contents(), PegLexer::<false>::new())
            .map_err(|it| TokenizationError(it))?;
        let tokens = tokenizer.into_vec();
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
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        output_path: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        use kodept_core::file_relative::CodePath;

        let source_name = match source.path() {
            CodePath::ToFile(x) => x
                .file_name()
                .expect("Source should be a file")
                .to_os_string(),
            CodePath::ToMemory(x) => x.into(),
        };
        let file_output_path = output_path.join(source_name);
        match self.option {
            InspectingOptions::Tokenizer => self
                .inspect_tokenizer(&source, &file_output_path)
                .or_emit(settings, &source, source.path())?,
            InspectingOptions::Parser => self.inspect_parser(&source, &file_output_path).or_emit(
                settings,
                &source,
                source.path(),
            )?,
            InspectingOptions::Both => {
                self.inspect_tokenizer(&source, &file_output_path).or_emit(
                    settings,
                    &source,
                    source.path(),
                )?;
                self.inspect_parser(&source, &file_output_path).or_emit(
                    settings,
                    &source,
                    source.path(),
                )?
            }
        };
        Ok(())
    }
}
