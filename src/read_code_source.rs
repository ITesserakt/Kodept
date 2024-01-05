use std::borrow::Cow;
use std::env::current_dir;
use std::io::Read;

use codespan_reporting::files::line_starts;
use size_of::SizeOf;

use kodept_core::code_point::CodePoint;
use kodept_core::code_source::CodeSource;
use kodept_core::file_relative::CodePath;
use kodept_core::structure::span::CodeHolder;

#[derive(SizeOf, Debug)]
pub struct ReadCodeSource {
    source_contents: String,
    source_path: CodePath,
    line_starts: Vec<usize>,
}

impl ReadCodeSource {
    pub fn path(&self) -> CodePath {
        self.source_path.clone()
    }

    pub fn contents(&self) -> &str {
        &self.source_contents
    }

    pub(crate) fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }

    pub fn into_inner(self) -> (String, CodePath) {
        (self.source_contents, self.source_path)
    }
}

impl TryFrom<CodeSource> for ReadCodeSource {
    type Error = std::io::Error;

    fn try_from(value: CodeSource) -> Result<Self, Self::Error> {
        let path = value.path().get_relative_path(&current_dir()?);
        match value {
            CodeSource::Memory { contents, .. } => {
                let starts = line_starts(contents.get_ref()).collect();
                Ok(Self {
                    source_contents: contents.into_inner(),
                    source_path: path,
                    line_starts: starts,
                })
            }
            CodeSource::File { mut file, .. } => {
                let mut buf = String::with_capacity(1024);
                file.read_to_string(&mut buf)?;
                let starts = line_starts(&buf).collect();
                Ok(Self {
                    source_contents: buf,
                    source_path: path,
                    line_starts: starts,
                })
            }
        }
    }
}

impl CodeHolder for ReadCodeSource {
    fn get_chunk(&self, at: CodePoint) -> Cow<str> {
        Cow::Borrowed(&self.source_contents[at.as_range()])
    }
}
