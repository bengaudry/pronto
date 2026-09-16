use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

const GIT_DIR: &str = ".git";
pub const PRONTO_DIR: &str = ".pronto";

pub fn find_project_root(target_path: Option<&Path>) -> PathBuf {
    let mut current: PathBuf;

    match target_path {
        Some(path) => { current = path.parent().unwrap_or(Path::new(".")).to_path_buf(); },
        None => { current = Path::new(".").to_path_buf(); }
    }

    let initial_path = current.clone();

    loop {
        // if .git in current, it is the root of the project
        if current.join(GIT_DIR).is_dir() {
            return current;
        }

        // if .pronto already exists
        if current.join(PRONTO_DIR).is_dir() {
            return current;
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            // If we reached the root without finding anything, we consider that the folder containing
            // the .c is the root, or the direct parent folder if it is named "src"
            let default_path = initial_path.parent().unwrap_or(Path::new("."));
            if default_path.file_name().map_or(false, |name| name == "src") {
                return default_path.parent().unwrap_or(default_path).to_path_buf();
            }
            return default_path.to_path_buf();
        }
    }
}

pub fn get_pronto_dir(target_path: Option<&Path>) -> PathBuf {
    let project_root = find_project_root(target_path);
    project_root.join(PRONTO_DIR)
}

pub fn get_build_dir(target_path: Option<&Path>) -> PathBuf {
    get_pronto_dir(target_path).join("build")
}

pub fn create_build_dir_in_curr_dir_if_not_exists() -> Result<PathBuf, Error> {
    let build_dir = Path::new(".").join(PRONTO_DIR).join("build");
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

pub fn create_build_dir_if_not_exists(target_path: Option<&Path>) -> Result<PathBuf, Error> {
    let build_dir = get_build_dir(target_path);
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
