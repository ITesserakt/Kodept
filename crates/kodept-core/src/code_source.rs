use mmap_rs::{Mmap, MmapFlags, MmapOptions};
use std::fs::{File};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum CodeSourceError {
    MMapError(#[from] mmap_rs::Error),
    IO(#[from] std::io::Error)
}

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
        size: u64
    },
}

impl CodeSource {
    pub fn memory(contents: String) -> Self {
        Self::Memory {
            contents: Cursor::new(contents),
        }
    }

    pub fn file<S: Into<PathBuf>>(name: S, contents: File) -> Self {
        Self::File {
            name: name.into(),
            file: contents,
        }
    }

    #[allow(unsafe_code)]
    pub fn mmap<S: Into<PathBuf>>(name: S, mut file: File, size: Option<u64>) -> Result<Self, CodeSourceError> {
        let size = match size {
            None => {
                let size = file.seek(SeekFrom::End(0))?;
                file.rewind()?;
                size
            }
            Some(x) => x
        };
        // SAFETY: compiler is not going to modify this file.
        // But any other app can, and here we're not checking that.
        let options = unsafe {
            MmapOptions::new(size as usize)?
                .with_flags(MmapFlags::SEQUENTIAL | MmapFlags::SHARED)
                .with_file(&file, 0)
        };
        let map = options.map()?;

        Ok(Self::MappedFile {
            name: name.into(),
            map: Cursor::new(map),
            size
        })
    }
}

impl Read for CodeSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            CodeSource::Memory { contents, .. } => contents.read(buf),
            CodeSource::File { file, .. } => file.read(buf),
            CodeSource::MappedFile { map, .. } => map.read(buf),
        }
    }
}

impl Seek for CodeSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            CodeSource::Memory { contents, .. } => contents.seek(pos),
            CodeSource::File { file, .. } => file.seek(pos),
            CodeSource::MappedFile { map, .. } => map.seek(pos),
        }
    }
}
