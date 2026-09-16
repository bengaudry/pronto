use crate::build_dir::compiler::compile_object_recursively;
use crate::project::ensure_build_dir;
use crate::project::executables::add_executable;
use crate::toolchain::gcc::{invoke_gcc, is_gcc_available};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn link_target(target: String, cflags: &[String]) -> anyhow::Result<PathBuf> {
    if !target.ends_with(".c") {
        anyhow::bail!("Expected a C file as argument.");
    }

    if !is_gcc_available() {
        anyhow::bail!("gcc not available.");
    }

    // Create the .pronto dir at the cwd
    let build_path = ensure_build_dir(Some(Path::new(&target)))
        .map_err(|err| anyhow::anyhow!("Could not create .pronto folder. Error : {}", err))?;

    let target_path = Path::new(&target);

    let mut objects = compile_object_recursively(
        target_path.to_path_buf(),
        build_path,
        &mut HashSet::new(),
        cflags,
    )?;

    // Build final executable
    let executable_path = target_path.with_extension("");
    objects.extend(cflags.iter().cloned());
    objects.push("-o".to_string());
    objects.push(
        executable_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Could not convert path into string"))?
            .to_string(),
    );
    // GccError propagates untouched so main() can detect it via downcast.
    invoke_gcc(objects)?;
    add_executable(executable_path.to_path_buf(), Some(target_path))?;
    println!("\nBuilt executable at path : {:?}", executable_path);

    Ok(executable_path)
}
