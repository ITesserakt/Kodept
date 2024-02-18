use std::fmt::Formatter;
use std::path::{Path, PathBuf};

use crate::code_source::CodeSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodePath {
    ToFile(PathBuf),
    ToMemory(String),
}

#[derive(Debug)]
pub struct FileRelative<T> {
    pub value: T,
    pub filepath: CodePath,
}

impl CodeSource {
    pub fn with_filename<T>(&self, f: impl Fn(&Self) -> T) -> FileRelative<T> {
        FileRelative {
            value: f(self),
            filepath: self.path(),
        }
    }

    #[must_use]
    pub fn path(&self) -> CodePath {
        match self {
            CodeSource::Memory { name, .. } => CodePath::ToMemory(name.clone()),
            CodeSource::File { name, .. } => CodePath::ToFile(name.clone()),
        }
    }
}

impl<T> FileRelative<T> {
    pub fn map<V>(self, f: impl FnOnce(T) -> V) -> FileRelative<V> {
        FileRelative {
            value: f(self.value),
            filepath: self.filepath,
        }
    }
}

impl CodePath {
    pub fn get_relative_path<P: AsRef<Path> + ?Sized>(&self, base: &P) -> CodePath {
        match self {
            CodePath::ToFile(p) => {
                CodePath::ToFile(pathdiff::diff_paths(p, base).unwrap_or(p.clone()))
            }
            CodePath::ToMemory(s) => CodePath::ToMemory(s.clone()),
        }
    }
}

impl std::fmt::Display for CodePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CodePath::ToFile(p) => write!(f, "{0}", p.display()),
            CodePath::ToMemory(p) => write!(f, "{p}"),
        }
    }
}
