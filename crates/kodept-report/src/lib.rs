use kodept_core::file_name::FileName;
use tracing::warn;

pub mod error;

pub fn warn_about_broken_rlt<T>() {
    warn!(
        expected = std::any::type_name::<T>(),
        "Skipping some checks because node in RLT either doesn't exist or has different type."
    );
}

pub type FileId = u16;

#[derive(Debug)]
pub struct FileDescriptor {
    pub name: FileName,
    pub id: FileId,
}
