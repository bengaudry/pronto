use std::fmt;
use std::path::PathBuf;
use std::process::Command;

/// A gcc invocation that exited with a non-zero status (i.e. a compile/link
/// error in the user's C code). This is *not* a pronto bug, so callers must
/// surface it without the "open a GitHub issue" footer.
#[derive(Debug)]
pub struct GccError {
    pub code: Option<i32>,
    pub stderr: String,
}

impl fmt::Display for GccError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stderr = self.stderr.trim();
        if stderr.is_empty() {
            match self.code {
                Some(code) => write!(f, "gcc failed with exit code {}", code),
                None => write!(f, "gcc failed"),
            }
        } else {
            write!(f, "{}", stderr)
        }
    }
}

impl std::error::Error for GccError {}

pub fn is_gcc_available() -> bool {
    let check_cmd_status = Command::new("gcc")
        .arg("--version")
        .status()
        .expect("gcc command not found");

    check_cmd_status.success()
}

pub fn invoke_gcc(args: Vec<String>) -> anyhow::Result<()> {
    let output = Command::new("gcc")
        .args(&args)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to spawn gcc: {}", e))?;

    if !output.status.success() {
        return Err(GccError {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        }
        .into());
    }

    Ok(())
}

pub fn compile_to_object_with_dependencies(
    target_path: PathBuf,
    build_path: PathBuf,
) -> anyhow::Result<PathBuf> {
    // Create the path to the mirrored target in the .pronto dir
    let target_path_in_build_dir = build_path.join(&target_path);
    let target_file_o = target_path_in_build_dir.with_extension("o");
    let parent_dir = target_file_o.parent().expect("Could not get parent dir");
    std::fs::create_dir_all(parent_dir).expect("Could not create subdirectories");

    return invoke_gcc(Vec::from([
        "-MMD".to_string(),
        target_path.to_str().expect("").to_string(),
        "-c".to_string(),
        "-o".to_string(),
        target_file_o.to_str().expect("").to_string(),
    ]))
    .and(Ok(target_file_o));
}
