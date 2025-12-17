use bevy_ecs::prelude::Resource;
use kodept_core::file_name::FileName;
use std::ffi::OsStr;
use std::fs::create_dir_all;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Resource)]
pub struct OutputDirectory {
    path: PathBuf,
}

#[derive(Debug, Resource)]
pub enum Lexer {
    Peg,
    PegWithTracing,
    Ascii,
}

#[derive(Debug, Resource)]
pub enum Parser {
    Peg,
}

impl OutputDirectory {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn create_missing_folders(&self) -> std::io::Result<()> {
        match create_dir_all(&self.path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn get_path_for_source<Q: AsRef<OsStr> + ?Sized>(
        &self,
        source: &FileName,
        extension: &Q,
    ) -> std::io::Result<PathBuf> {
        self.create_missing_folders()?;
        let new_path = source.build_file_path().with_extension(extension.as_ref());
        let name = new_path.file_name().unwrap();
        Ok(self.path.join(name))
    }
}
