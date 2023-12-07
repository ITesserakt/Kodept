use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;

#[cfg(feature = "size-of")]
use size_of::Context;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(Debug)]
pub enum CodeSource {
    Memory {
        name: String,
        contents: Cursor<String>,
    },
    File {
        name: PathBuf,
        file: File,
    },
}

impl CodeSource {
    pub fn memory<S: Into<String>>(name: S, contents: String) -> Self {
        Self::Memory {
            name: name.into(),
            contents: Cursor::new(contents),
        }
    }

    pub fn file<S: Into<PathBuf>>(name: S, contents: File) -> Self {
        Self::File {
            name: name.into(),
            file: contents,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            CodeSource::Memory { name, .. } => name,
            CodeSource::File { name, .. } => name.to_str().unwrap_or("<unknown location>"),
        }
    }
}

impl Read for CodeSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            CodeSource::Memory { contents, .. } => contents.read(buf),
            CodeSource::File { file, .. } => file.read(buf),
        }
    }
}

impl Seek for CodeSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            CodeSource::Memory { contents, .. } => contents.seek(pos),
            CodeSource::File { file, .. } => file.seek(pos),
        }
    }
}

#[cfg(feature = "size-of")]
impl SizeOf for CodeSource {
    fn size_of_children(&self, context: &mut Context) {
        match self {
            CodeSource::Memory { name, contents } => {
                name.size_of_children(context);
                contents.get_ref().size_of_children(context);
            }
            CodeSource::File { name, .. } => {
                name.size_of_children(context);
            }
        }
    }
}
