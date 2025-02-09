use std::borrow::Cow;
use std::fmt::Formatter;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Instant};
use derive_more::Constructor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u16);

#[derive(Debug, Clone, Eq, Constructor)]
pub struct FileDescriptor {
    name: FileName,
    id: FileId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileName {
    Real(PathBuf),
    Anon,
    Custom(Cow<'static, str>)
}

impl FileId {
    pub fn generate() -> Self {
        static ID_GENERATOR: AtomicU16 = AtomicU16::new(0);
        let id = ID_GENERATOR.fetch_add(1, SeqCst);
        FileId(id)
    }
}

impl PartialEq for FileDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for FileDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl FileDescriptor {
    pub fn name(&self) -> &FileName {
        &self.name
    }

    pub fn id(&self) -> FileId {
        self.id
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
