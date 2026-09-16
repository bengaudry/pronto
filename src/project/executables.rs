use super::resolve_pronto_dir;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

pub const EXECUTABLES_FILENAME: &str = "executables.txt";

/// Registry lives next to the build dir (`<project>/.pronto/executables.txt`),
/// resolved from the target so `run`/`compile` and `clean` share one location.
pub fn get_executables_list_file_path(target_path: Option<&Path>) -> PathBuf {
    resolve_pronto_dir(target_path).join(EXECUTABLES_FILENAME)
}

pub fn load_executables_registry(
    target_path: Option<&Path>,
) -> anyhow::Result<Vec<PathBuf>> {
    let fp = get_executables_list_file_path(target_path);
    if !fp.exists() || !fp.is_file() {
        return Ok(Vec::new());
    }

    let file = File::open(&fp)
        .map_err(|e| anyhow::anyhow!("Could not open {}: {}", fp.display(), e))?;

    let mut executables_list: Vec<PathBuf> = Vec::new();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line =
            line.map_err(|e| anyhow::anyhow!("Could not read {}: {}", fp.display(), e))?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            executables_list.push(Path::new(trimmed).to_path_buf());
        }
    }

    Ok(executables_list)
}

pub fn add_executable(
    executable_path: PathBuf,
    target_path: Option<&Path>,
) -> anyhow::Result<()> {
    if !executable_path.is_file() {
        anyhow::bail!(
            "Could not find executable file at {}",
            executable_path.display()
        );
    }

    let mut executables_list = load_executables_registry(target_path)?;
    let canonical = executable_path.canonicalize().map_err(|e| {
        anyhow::anyhow!("Could not canonicalize {}: {}", executable_path.display(), e)
    })?;

    if !executables_list.contains(&canonical) {
        executables_list.push(canonical);
        persist_executables_registry(executables_list, target_path)?;
    }
    Ok(())
}

pub fn remove_executable(
    executable_path: PathBuf,
    target_path: Option<&Path>,
) -> anyhow::Result<()> {
    let mut executables_list = load_executables_registry(target_path)?;
    executables_list.retain(|p| p != &executable_path);
    persist_executables_registry(executables_list, target_path)
}

pub fn persist_executables_registry(
    executables_list: Vec<PathBuf>,
    target_path: Option<&Path>,
) -> anyhow::Result<()> {
    let fp = get_executables_list_file_path(target_path);
    if fp.exists() && !fp.is_file() {
        anyhow::bail!("{} already exists, and is not a file", fp.display());
    }

    // The build dir is created by ensure_build_dir, but the registry sits in
    // `.pronto/` itself: make sure the parent exists before File::create,
    // otherwise a missing dir surfaces as "Could not open executables.txt".
    if let Some(parent) = fp.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            anyhow::anyhow!("Could not create {}: {}", parent.display(), e)
        })?;
    }

    let file =
        File::create(&fp).map_err(|e| anyhow::anyhow!("Could not open {}: {}", fp.display(), e))?;
    let mut writer = BufWriter::new(file);

    for exe in executables_list {
        if let Some(exe_str) = exe.to_str() {
            writeln!(writer, "{}", exe_str).map_err(|e| {
                anyhow::anyhow!("Failed to write executable path to {}: {}", fp.display(), e)
            })?;
        }
    }
    writer
        .flush()
        .map_err(|e| anyhow::anyhow!("Failed to flush {}: {}", fp.display(), e))?;
    Ok(())
}
