use derive_more::{Display, Error, From};
use kodept_core::file_name::FileName;
use kodept_frontend::prelude::{ReadSource, TryReadSource};
use memmap2::{Mmap, MmapOptions};
use std::env::current_dir;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug)]
pub enum CodeSource {
    Memory {
        contents: Cursor<String>,
    },
    File {
        name: PathBuf,
        file: File,
    },
    MappedFile {
        name: PathBuf,
        map: Cursor<Mmap>,
        size: u64,
    },
}

impl CodeSource {
    pub(crate) fn memory(contents: String) -> Self {
        Self::Memory {
            contents: Cursor::new(contents),
        }
    }

    pub(crate) fn file<S: Into<PathBuf>>(name: S, contents: File) -> Self {
        Self::File {
            name: name.into(),
            file: contents,
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn mmap<S: Into<PathBuf>>(
        name: S,
        mut file: File,
        size: Option<u64>,
    ) -> Result<Self, CodeSourceError> {
        let size = match size {
            None => {
                let size = file.seek(SeekFrom::End(0))?;
                file.rewind()?;
                size
            }
            Some(x) => x,
        };
        // SAFETY: compiler is not going to modify this file.
        // But any other app can, and here we're not checking that.
        let map = unsafe { MmapOptions::new().map(&file) }?;

        Ok(Self::MappedFile {
            name: name.into(),
            map: Cursor::new(map),
            size,
        })
    }

    #[must_use]
    pub fn path(&self) -> FileName {
        match self {
            CodeSource::Memory { .. } => FileName::Anon,
            CodeSource::File { name, .. } => FileName::Real(name.clone()),
            CodeSource::MappedFile { name, .. } => FileName::Real(name.clone()),
        }
    }
}

impl Read for CodeSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            CodeSource::Memory { contents } => contents.read(buf),
            CodeSource::File { file, .. } => file.read(buf),
            CodeSource::MappedFile { map, .. } => map.read(buf),
        }
    }
}

#[derive(Debug, Error, Display, From)]
pub enum CodeSourceError {
    IO(std::io::Error),
    UTF8Str(std::str::Utf8Error),
    UTF8String(std::string::FromUtf8Error),
}

impl TryReadSource for CodeSource {
    type Error = CodeSourceError;

    fn try_read(self) -> Result<ReadSource, Self::Error> {
        let path = self.path().get_relative_path(&current_dir()?);
        match self {
            CodeSource::Memory { contents, .. } => {
                Ok(ReadSource::explicit(contents.into_inner(), path))
            }
            CodeSource::File { mut file, .. } => {
                let mut buf = Vec::with_capacity(1024);
                file.read_to_end(&mut buf)?;
                let buf = String::from_utf8(buf)?;
                Ok(ReadSource::explicit(buf, path))
            }
            CodeSource::MappedFile { map, .. } => Ok(ReadSource::implicit(map.into_inner(), path)?),
        }
    }
}
