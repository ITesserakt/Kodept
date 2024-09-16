use std::borrow::Cow;
use std::fmt::Formatter;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Instant};
use crate::code_source::CodeSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileName {
    Real(PathBuf),
    Anon,
    Custom(Cow<'static, str>)
}

impl CodeSource {
    #[must_use]
    pub fn path(&self) -> FileName {
        match self {
            CodeSource::Memory { .. } => FileName::Anon,
            CodeSource::File { name, .. } => FileName::Real(name.clone()),
            CodeSource::MappedFile { name, .. } => FileName::Real(name.clone()),
        }
    }
}

impl FileName {
    pub fn get_relative_path<P: AsRef<Path> + ?Sized>(&self, base: &P) -> FileName {
        match self {
            FileName::Real(p) => {
                FileName::Real(pathdiff::diff_paths(p, base).unwrap_or(p.clone()))
            }
            FileName::Anon => FileName::Anon,
            FileName::Custom(c) => FileName::Custom(c.clone())
        }
    }
    
    fn generate_hash() -> u64 {
        let instant = Instant::now();
        let mut hasher = DefaultHasher::new();
        instant.hash(&mut hasher);
        hasher.finish()
    }
    
    pub fn build_file_path(&self) -> Cow<Path> {
        match self {
            FileName::Real(x) => Cow::Borrowed(x.as_path()),
            FileName::Anon => {
                let hash = Self::generate_hash();
                Cow::Owned(format!("__{hash}.kd").into())
            },
            FileName::Custom(c) => {
                let hash = Self::generate_hash();
                Cow::Owned(format!("__{hash}-{c}.kd").into())
            }
        }
    }
    
    pub fn to_string_lossy(&self) -> Cow<str> {
        match self {
            FileName::Real(x) => x.to_string_lossy(),
            FileName::Anon => "<anonymous>".into(),
            FileName::Custom(c) => Cow::Owned(format!("<{c}>")) 
        }
    }
}

impl std::fmt::Display for FileName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileName::Real(p) => write!(f, "{0}", p.display()),
            FileName::Anon => write!(f, "<anonymous>"),
            FileName::Custom(c) => write!(f, "<{c}>")
        }
    }
}
