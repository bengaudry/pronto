pub mod cache;
pub mod dependencies;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::build_dir::cache::is_source_newer_than_artifact;
use crate::toolchain::gcc::is_gcc_available;
use crate::build_dir::dependencies::{Dependency, parse_dot_d_dependencies};
use crate::toolchain::gcc::{compile_to_object_with_dependencies, invoke_gcc};
use crate::project::{create_build_dir_if_not_exists, PRONTO_DIR};
use crate::project::executables::{add_executable, load_executables_registry, remove_executable};

fn compile_object_recursively(
    target_path: PathBuf,
    build_path: PathBuf,
    visited: &mut HashSet<PathBuf>,
) -> Vec<String> {
    if visited.contains(&target_path) {
        return Vec::new();
    }
    visited.insert(target_path.clone());
    let target = target_path.file_name().unwrap();

    // Path to .o in .pronto dir
    let target_path_in_build_dir = build_path.join(&target_path);
    let target_file_o = target_path_in_build_dir.with_extension("o");
    let target_file_d = target_path_in_build_dir.with_extension("d");

    // Check if .o and .d already exists, and if the source .c file has been modified since
    let mut is_newer = true;
    if target_file_o.is_file() && target_file_d.is_file() {
        match is_source_newer_than_artifact(target_path.to_path_buf(), target_file_o.clone()) {
            Ok(newer_state) => is_newer = newer_state,
            Err(_) => {
                // TODO : Clean .pronto (corrupted) and retry
            }
        }
    }

    let mut objects: Vec<String> = Vec::new();

    // if c file has been modified since .o has been created
    if is_newer {
        // path to generated .o in the .pronto folder
        let object_file_path =
            compile_to_object_with_dependencies(target_path.to_path_buf(), build_path.to_path_buf())
                .expect("Could not use gcc for target.");
        let path_str = object_file_path
            .to_str()
            .expect("Invalid UTF-8")
            .to_string();
        objects.push(path_str);
        println!("Compiling {}...", target.to_str().unwrap())
    } else {
        objects.push(target_file_o.to_str().expect("Invalid UTF-8").to_string());
        println!(
            "{} has not changed, no need to recompile.",
            target.to_str().unwrap()
        )
    }

    // Analyse the .d file that has just been created, or already existed before
    match parse_dot_d_dependencies(target_file_d) {
        Ok(dependencies) => {
            for dependency in dependencies {
                match dependency {
                    Dependency::Header {
                        file: _,
                        source_file,
                    } => {
                        if source_file.is_some() {
                            objects.append(&mut compile_object_recursively(
                                source_file.unwrap(),
                                build_path.clone(),
                                visited,
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(_) => {
            // TODO : Handle error
        }
    }

    objects
}

pub fn link_target(target: String) -> anyhow::Result<PathBuf> {
    if !target.ends_with(".c") {
        anyhow::bail!("Expected a C file as argument.");
    }

    if !is_gcc_available() {
        anyhow::bail!("gcc not available.");
    }

    // Create the .pronto dir at the cwd
    let build_path = create_build_dir_if_not_exists(Some(Path::new(&target)))
        .map_err(|err| anyhow::anyhow!("Could not create .pronto folder. Error : {}", err))?;

    let target_path = Path::new(&target);

    let mut objects = compile_object_recursively(target_path.to_path_buf(), build_path, &mut HashSet::new());

    // Build final executable
    let executable_path = target_path.with_extension("");
    objects.push("-o".to_string());
    objects.push(
        executable_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Could not convert path into string"))?
            .to_string(),
    );
    invoke_gcc(objects).map_err(|e| anyhow::anyhow!("{}", e))?;
    add_executable(executable_path.to_path_buf());
    println!("\nBuilt executable at path : {:?}", executable_path);

    Ok(executable_path)
}

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
