use std::fs::create_dir;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

pub const BUILD_DIR: &str = ".pronto";

pub fn build_dir_exists() -> bool {
    let build_dir: &Path = Path::new(BUILD_DIR);
    build_dir.exists() && build_dir.is_dir()
}

pub fn create_build_dir_if_not_exists() -> Result<PathBuf, Error> {
    let build_dir: &Path = Path::new(BUILD_DIR);
    if build_dir.exists() {
        if !build_dir.is_dir() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("{} exists, but is not a folder.", BUILD_DIR),
            ));
        }
        return Ok(build_dir.to_path_buf());
    }
    
    create_dir(build_dir)?;
    
    Ok(build_dir.to_path_buf())
}
