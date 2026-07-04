use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

const GIT_DIR: &str = ".git";
pub const PRONTO_DIR: &str = ".pronto";

pub fn find_project_root(target_path: &Path) -> PathBuf {
    let mut current = target_path.parent().unwrap_or(Path::new(".")).to_path_buf();

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
            // Si on a atteint la racine du disque sans rien trouver,
            // on considère par défaut que le dossier contenant le .c est la racine,
            // ou le dossier parent direct s'il s'appelle "src"
            let default_path = target_path.parent().unwrap_or(Path::new("."));
            if default_path.file_name().map_or(false, |name| name == "src") {
                return default_path.parent().unwrap_or(default_path).to_path_buf();
            }
            return default_path.to_path_buf();
        }
    }
}

pub fn get_pronto_dir() -> PathBuf {
    Path::new(PRONTO_DIR).to_path_buf()
}

pub fn get_build_dir() -> PathBuf {
    get_pronto_dir().join("build")
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
