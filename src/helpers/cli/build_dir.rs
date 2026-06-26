use std::fs;
use std::io::{Error, ErrorKind};
use std::ops::Deref;
use std::path::{Path, PathBuf};

pub const PRONTO_DIR: &str = ".pronto";

pub fn get_build_dir() -> PathBuf {
    Path::new(PRONTO_DIR).join("build")
}

pub fn create_build_dir_if_not_exists() -> Result<PathBuf, Error> {
    let build_dir = get_build_dir();
    if build_dir.exists() {
        if !build_dir.is_dir() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("{} exists, but is not a folder.", build_dir.display()),
            ));
        }
        return Ok(build_dir.to_path_buf());
    }
    
    fs::create_dir_all(build_dir.clone())?;
    
    Ok(build_dir.to_path_buf())
}
