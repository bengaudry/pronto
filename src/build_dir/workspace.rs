use crate::project::{ensure_build_dir, resolve_build_dir, resolve_tmp_dir};
use anyhow::Context;
use std::fs;
use std::fs::{remove_dir_all, remove_file, rename};
use std::io::Result;
use std::path::{Path, PathBuf};

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir_all(src_path, dst_path)?;
        } else {
            fs::copy(src_path, dst_path)?;
        }
    }

    Ok(())
}

pub fn prepare_workspace(target_path: &Path) -> anyhow::Result<PathBuf> {
    let build_path =
        ensure_build_dir(Some(target_path)).context("Could not create .pronto folder")?;

    let tmp_dir_path = resolve_tmp_dir(Some(target_path));
    if tmp_dir_path.exists() {
        if tmp_dir_path.is_dir() {
            remove_dir_all(tmp_dir_path.clone())?;
        }
        remove_file(tmp_dir_path.clone())?;
    }

    copy_dir_all(build_path, &tmp_dir_path)
        .context("Failed to duplicate existing build folder.")?;

    Ok(tmp_dir_path)
}

pub fn rollback_workspace(target_path: &Path) -> Result<()> {
    let tmp_dir_path = resolve_tmp_dir(Some(target_path));
    remove_dir_all(tmp_dir_path)
}

pub fn commit_workspace(target_path: &Path) -> Result<()> {
    let tmp_dir_path = resolve_tmp_dir(Some(target_path));
    let build_dir_path = resolve_build_dir(Some(target_path));
    remove_dir_all(build_dir_path.clone())?;
    rename(tmp_dir_path, build_dir_path)
}
