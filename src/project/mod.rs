pub mod executables;
pub mod gitignore;

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

pub fn resolve_pronto_dir(target_path: Option<&Path>) -> PathBuf {
    let project_root = find_project_root(target_path);
    project_root.join(PRONTO_DIR)
}

pub fn resolve_build_dir(target_path: Option<&Path>) -> PathBuf {
    resolve_pronto_dir(target_path).join("build")
}

/// Garantit que le répertoire de build `.pronto/build` existe.
///
/// Résout le chemin via `get_build_dir(target_path)` (donc via `find_project_root`),
/// puis le crée récursivement si nécessaire. Idempotente : si le dossier existe déjà,
/// le retourne sans erreur. Si un fichier non-dossier existe à cet emplacement,
/// renvoie `ErrorKind::AlreadyExists`.
///
/// # Arguments
/// * `target_path` - Chemin du fichier cible (ex: `Some(Path::new("src/main.c"))`)
///   pour résoudre la racine projet, ou `None` pour résoudre depuis `"."` (cwd).
///
/// # Errors
/// Renvoie une erreur si la création échoue (permissions, etc.) ou si le chemin
/// existe mais n'est pas un dossier.
pub fn ensure_build_dir(target_path: Option<&Path>) -> Result<PathBuf, Error> {
    let build_dir = resolve_build_dir(target_path);
    if build_dir.exists() {
        if !build_dir.is_dir() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("{} exists, but is not a folder.", build_dir.display()),
            ));
        }
        return Ok(build_dir);
    }

    fs::create_dir_all(&build_dir)?;

    Ok(build_dir)
}

pub fn create_build_dir_in_curr_dir_if_not_exists() -> Result<PathBuf, Error> {
    ensure_build_dir(None)
}

pub fn create_build_dir_if_not_exists(target_path: Option<&Path>) -> Result<PathBuf, Error> {
    ensure_build_dir(target_path)
}
