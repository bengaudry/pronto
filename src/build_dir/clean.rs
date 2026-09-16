use std::fs;

use crate::project::executables::{load_executables_registry, remove_executable};
use crate::project::resolve_pronto_dir;

pub fn clean_executables(full: bool) -> anyhow::Result<()> {
    let pronto_dir = resolve_pronto_dir(None);
    if !pronto_dir.is_dir() {
        anyhow::bail!(
            "The .pronto folder was not found. Run this command in the root directory where the .pronto folder is located."
        );
    }

    for exec_file_path in load_executables_registry(None)? {
        if exec_file_path.exists() && exec_file_path.is_file() {
            fs::remove_file(&exec_file_path).map_err(|e| {
                anyhow::anyhow!("Failed to remove executable {:?}: {}", exec_file_path, e)
            })?;
        } else {
            remove_executable(exec_file_path, None)?;
        }
    }

    if full {
        fs::remove_dir_all(&pronto_dir)
            .map_err(|e| anyhow::anyhow!("Failed to remove .pronto directory: {}", e))?;
    }

    Ok(())
}
