use std::fs;
use std::path::{Path};

use crate::project::{PRONTO_DIR};
use crate::project::executables::{load_executables_registry, remove_executable};

pub fn clean_executables(full: bool) -> anyhow::Result<()> {
    if !Path::new(PRONTO_DIR).is_dir() {
        anyhow::bail!("The .pronto folder was not found. Run this command in the root directory where the .pronto folder is located.");
    }

    for exec_file_path in load_executables_registry() {
        if exec_file_path.exists() && exec_file_path.is_file() {
            fs::remove_file(&exec_file_path)
                .map_err(|e| anyhow::anyhow!("Failed to remove executable {:?}: {}", exec_file_path, e))?;
        } else {
            remove_executable(exec_file_path);
        }
    }

    if full {
        fs::remove_dir_all(Path::new(PRONTO_DIR))
            .map_err(|e| anyhow::anyhow!("Failed to remove .pronto directory: {}", e))?;
    }

    Ok(())
}
