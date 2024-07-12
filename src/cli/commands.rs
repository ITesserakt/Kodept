use crate::cli::configs::CompilationConfig;
use crate::WideError;
use clap::{Parser, Subcommand, ValueEnum};
use codespan_reporting::diagnostic::Diagnostic;
use derive_more::Display;
use extend::ext;
use kodept::codespan_settings::CodespanSettings;
use kodept::codespan_settings::ReportExt;
use kodept::macro_context::{DefaultContext, ErrorReported};
use kodept::parse_error::ParseErrortExt;
use kodept::read_code_source::ReadCodeSource;
use kodept::steps::common;
use kodept::steps::common::Config;
use kodept_ast::ast_builder::ASTBuilder;
use kodept_core::file_relative::CodePath;
use kodept_core::structure::rlt::RLT;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_parse::error::parse_from_top;
use kodept_parse::token_stream::TokenStream;
use kodept_parse::tokenizer::Tokenizer;
#[cfg(feature = "parallel")]
use rayon::prelude::ParallelIterator;
use std::fs::{create_dir_all, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, ValueEnum, Clone, Display)]
enum InspectingOptions {
    Tokenizer,
    Parser,
    Both,
}

#[derive(Debug, Parser)]
pub struct InspectParser {
    /// Controls which step will be traced
    #[arg(default_value = "both", long = "inspect")]
    option: InspectingOptions,
    /// Additionally launch `pegviz` to produce html output
    #[arg(default_value_t = true, short = 'p', long = "pegviz")]
    use_pegviz: bool,
}

#[derive(Parser, Debug)]
pub struct Graph;

#[derive(Debug)]
pub struct Execute;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Output AST in .dot format
    Graph(Graph),
    /// Output parsing process files
    InspectParser(InspectParser),
}

#[ext]
impl<T> Result<T, Vec<Diagnostic<()>>> {
    fn or_emit_diagnostics(
        self,
        settings: &mut CodespanSettings,
        source: &ReadCodeSource,
    ) -> Result<T, ErrorReported> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                e.into_iter()
                    .try_for_each(|it| it.emit(settings, source))
                    .expect("Cannot emit diagnostics");
                Err(ErrorReported)
            }
        }
    }
}

impl Commands {
    fn build_rlt(source: &ReadCodeSource) -> Result<RLT, Vec<Diagnostic<()>>> {
        let tokenizer = Tokenizer::new(source.contents());
        let tokens = tokenizer.into_vec();
        let token_stream = TokenStream::new(&tokens);
        let result = parse_from_top(token_stream).map_err(|es| {
            es.into_iter()
                .map(|it| it.to_diagnostic())
                .collect::<Vec<_>>()
        })?;
        Ok(result)
    }

    fn ensure_path_exists(path: &Path) -> Result<(), WideError> {
        match create_dir_all(path) {
            Err(e) if e.kind() != ErrorKind::AlreadyExists => Err(e)?,
            _ => Ok(()),
        }
    }
}

impl Execute {
    #[cfg(feature = "parallel")]
    pub fn exec(
        self,
        sources: impl ParallelIterator<Item = ReadCodeSource>,
        settings: CodespanSettings,
        compilation_config: CompilationConfig,
    ) -> Result<(), WideError> {
        let config = into(compilation_config);
        sources.try_for_each_with(settings, move |settings, source| {
            self.exec_for_source(source, settings, &config)
        })
    }

    #[cfg(not(feature = "parallel"))]
    pub fn exec(
        self,
        sources: impl Iterator<Item = ReadCodeSource>,
        mut settings: CodespanSettings,
        compilation_config: CompilationConfig,
    ) -> Result<(), WideError> {
        let config = into(compilation_config);
        for source in sources {
            self.exec_for_source(source, &mut settings, &config)?;
        }
        Ok(())
    }

    fn exec_for_source(
        &self,
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        config: &Config,
    ) -> Result<(), WideError> {
        let rlt = Commands::build_rlt(&source)
            .or_emit_diagnostics(settings, &source)?
            .0;
        let (tree, accessor) = ASTBuilder.recursive_build(&rlt, &source);
        let mut context = DefaultContext::new(
            source.with_filename(|_| ReportCollector::new()),
            accessor,
            tree.build(),
        );
        common::run_common_steps(&mut context, config).or_else(|error| {
            error.unwrap_report().emit(settings, &source)?;
            Result::<_, WideError>::Ok(())
        })?;
        context.emit_diagnostics(settings, &source);

        Ok(())
    }
}

impl Graph {
    #[cfg(feature = "parallel")]
    pub fn exec(
        sources: impl ParallelIterator<Item = ReadCodeSource>,
        settings: CodespanSettings,
        output_path: PathBuf,
    ) -> Result<(), WideError> {
        sources.try_for_each_with(settings, |settings, source| {
            Graph::exec_for_source(source, settings, &output_path)
        })
    }

    #[cfg(not(feature = "parallel"))]
    pub fn exec(
        sources: impl Iterator<Item = ReadCodeSource>,
        mut settings: CodespanSettings,
        output_path: PathBuf,
    ) -> Result<(), WideError> {
        for source in sources {
            Self::exec_for_source(source, &mut settings, &output_path)?;
        }
        Ok(())
    }

    fn exec_for_source(
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        output_path: &Path,
    ) -> Result<(), WideError> {
        let rlt = Commands::build_rlt(&source).or_emit_diagnostics(settings, &source)?;
        let (tree, _) = ASTBuilder.recursive_build(&rlt.0, &source);
        let mut output_file = Self::get_output_file(&source, output_path)?;

        write!(output_file, "{}", tree.export_dot(&[]))?;
        Ok(())
    }

    fn get_output_file(source: &ReadCodeSource, output_path: &Path) -> Result<File, WideError> {
        let filename = match source.path() {
            CodePath::ToFile(file) => file
                .with_extension("kd.dot")
                .file_name()
                .expect("Source should be a file")
                .to_os_string(),
            CodePath::ToMemory(name) => PathBuf::from(name).with_extension("kd.dot").into(),
        };
        Commands::ensure_path_exists(output_path)?;
        Ok(File::create(output_path.join(filename))?)
    }
}

#[cfg(not(all(feature = "trace")))]
impl InspectParser {
    #[cfg(feature = "parallel")]
    pub fn exec(
        self,
        _sources: impl ParallelIterator<Item = ReadCodeSource>,
        _output_path: PathBuf,
    ) -> Result<(), WideError> {
        Err(anyhow::anyhow!(
            "Program is compiled without inspecting support"
        ))
    }

    #[cfg(not(feature = "parallel"))]
    pub fn exec(
        self,
        _sources: impl Iterator<Item = ReadCodeSource>,
        _output_path: PathBuf,
    ) -> Result<(), WideError> {
        Err(anyhow::anyhow!(
            "Program is compiled without inspecting support"
        ))
    }
}

#[cfg(feature = "trace")]
impl InspectParser {
    #[cfg(feature = "parallel")]
    pub fn exec(
        self,
        sources: impl ParallelIterator<Item = ReadCodeSource>,
        output_path: PathBuf,
    ) -> Result<(), WideError> {
        let output_path = output_path.join("debug");
        Commands::ensure_path_exists(&output_path)?;
        sources.try_for_each(|source| self.exec_for_source(source, &output_path))
    }

    #[cfg(not(feature = "parallel"))]
    pub fn exec(
        self,
        sources: impl Iterator<Item = ReadCodeSource>,
        output_path: PathBuf,
    ) -> Result<(), WideError> {
        let output_path = output_path.join("debug");
        Commands::ensure_path_exists(&output_path)?;
        for source in sources {
            self.exec_for_source(source, &output_path)?
        }
        Ok(())
    }

    fn launch_pegviz<P: AsRef<Path>>(&self, input_file_path: P) -> Result<(), WideError> {
        use std::process::{Command, Output};
        use tracing::{info, warn, debug, error};

        if !self.use_pegviz {
            return Ok(());
        }

        let output_path = input_file_path.as_ref().with_extension("html");
        let input_file = File::open(input_file_path)?;
        let Output { status, stdout, ..} = Command::new("pegviz")
            .args(["--output".into(), output_path])
            .stdin(input_file)
            .output()?;
        let stdout = String::from_utf8(stdout)?;
        stdout.lines().for_each(|line| {
            match line.split_once(":") {
                None if line.starts_with("= pegviz generated to") => info!("{}", line.strip_prefix("= ").unwrap()),
                None => debug!("{line}"),
                Some((a, _)) if a.contains("error") => error!("{line}"),
                Some(_) => debug!("{line}")
            }
        });

        match status.code() {
            None => warn!("`pegviz` exited by signal"),
            Some(0) => {},
            Some(code) => Err(anyhow::anyhow!(
                "`pegviz` exited with non-zero exit code: {code}",
            ))?,
        }
        Ok(())
    }

    fn exec_for_source(&self, source: ReadCodeSource, output_path: &Path) -> Result<(), WideError> {
        let source_name = match source.path() {
            CodePath::ToFile(x) => x
                .file_name()
                .expect("Source should be a file")
                .to_os_string(),
            CodePath::ToMemory(x) => x.into(),
        };
        let file_output_path = output_path.join(source_name);
        match self.option {
            InspectingOptions::Tokenizer => {
                use kodept_parse::tokenizer::TracedTokenizer;
                let file = File::create(file_output_path.with_extension("tok.peg"))?;
                let _gag = gag::Redirect::stdout(file)?;
                TracedTokenizer::try_new(source.contents())?;

                self.launch_pegviz(file_output_path.with_extension("tok.peg"))?;
            }
            InspectingOptions::Parser => {
                let tokenizer = Tokenizer::try_new(source.contents())?;
                let tokens = tokenizer.into_vec();
                let tokens = TokenStream::new(&tokens);

                let file = File::create(file_output_path.with_extension("par.peg"))?;
                let _gag = gag::Redirect::stdout(file)?;
                let _ = parse_from_top(tokens);

                self.launch_pegviz(file_output_path.with_extension("par.peg"))?;
            }
            InspectingOptions::Both => {
                use kodept_parse::tokenizer::TracedTokenizer;
                let tok_file = File::create(file_output_path.with_extension("tok.peg"))?;
                let tokens = {
                    let _gag = gag::Redirect::stdout(tok_file)?;
                    let tokenizer = TracedTokenizer::try_new(source.contents())?;
                    tokenizer.into_vec()
                };

                let parse_file = File::create(file_output_path.with_extension("par.peg"))?;
                {
                    let _gag1 = gag::Redirect::stdout(parse_file)?;
                    let _ = parse_from_top(TokenStream::new(&tokens));
                }

                self.launch_pegviz(file_output_path.with_extension("tok.peg"))?;
                self.launch_pegviz(file_output_path.with_extension("par.peg"))?;
            }
        };
        Ok(())
    }
}

fn into(config: CompilationConfig) -> Config {
    Config::new(config.type_checking_recursion_depth)
}
