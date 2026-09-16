use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::PathBuf;
use crate::build_dir::cache::is_source_newer_than_artifact;
use crate::toolchain::gcc::{compile_to_object_with_dependencies};
use crate::build_dir::dependencies::{Dependency, parse_dot_d_dependencies};

fn needs_recompilation(target_path: PathBuf, target_file_o: PathBuf, target_file_d: PathBuf) -> bool {
    // Check if .o and .d already exists, and if the source .c file has been modified since
    let mut is_newer = true;
    if target_file_o.is_file() && target_file_d.is_file() {
        match is_source_newer_than_artifact(target_path, target_file_o.clone()) {
            Ok(newer_state) => is_newer = newer_state,
            Err(_) => {
                // TODO : Clean .pronto (corrupted) and retry
            }
        }
    }
    is_newer
}

fn compile_single_object(
    target: &OsStr,
    target_path: PathBuf,
    build_path: PathBuf,
) -> anyhow::Result<String> {
    // path to generated .o in the .pronto folder
    // A gcc failure here is a user code error (GccError), propagated as-is
    // so main() can render it without the GitHub footer.
    let object_file_path = compile_to_object_with_dependencies(target_path, build_path)?;
    let path_str = object_file_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in object path"))?
        .to_string();
    println!("Compiling {}...", target.to_str().unwrap_or("?"));
    Ok(path_str)
}

pub fn compile_object_recursively(
    target_path: PathBuf,
    build_path: PathBuf,
    visited: &mut HashSet<PathBuf>,
) -> anyhow::Result<Vec<String>> {
    if visited.contains(&target_path) {
        return Ok(Vec::new());
    }
    visited.insert(target_path.clone());
    let target = target_path.file_name().unwrap();

    // Path to .o in .pronto dir
    let target_path_in_build_dir = build_path.join(&target_path);
    let target_file_o = target_path_in_build_dir.with_extension("o");
    let target_file_d = target_path_in_build_dir.with_extension("d");

    let mut objects: Vec<String> = Vec::new();

    // if c file has been modified since .o has been created
    if needs_recompilation(target_path.clone(), target_file_o.clone(), target_file_d.clone()) {
        let path_str =
            compile_single_object(target, target_path.to_path_buf(), build_path.to_path_buf())?;
        objects.push(path_str);
    } else {
        objects.push(
            target_file_o
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in object path"))?
                .to_string(),
        );
        println!(
            "{} has not changed, no need to recompile.",
            target.to_str().unwrap_or("?")
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
                        if let Some(source_file) = source_file {
                            objects.append(&mut compile_object_recursively(
                                source_file,
                                build_path.clone(),
                                visited,
                            )?);
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

    Ok(objects)
}
