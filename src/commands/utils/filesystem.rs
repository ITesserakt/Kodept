use std::ffi::OsStr;
use std::fs::{create_dir_all, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn require_output_file<P: AsRef<Path> + ?Sized, Q: AsRef<OsStr> + ?Sized>(
    base_path: &P,
    base_file_name: &Q,
    extension: &str,
) -> std::io::Result<(File, PathBuf)> {
    ensure_path_exists(base_path.as_ref())?;
    let file_path = base_path
        .as_ref()
        .join(base_file_name.as_ref())
        .with_extension(extension);
    Ok((File::create(&file_path)?, file_path))
}

fn ensure_path_exists(path: &Path) -> std::io::Result<()> {
    match create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e),
    }
}
