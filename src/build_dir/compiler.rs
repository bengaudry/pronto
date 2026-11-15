use crate::build_dir::cache::is_source_newer_than_artifact;
use crate::build_dir::dependencies::{Dependency, parse_dot_d_dependencies};
use crate::build_dir::workspace::{commit_workspace, prepare_workspace, rollback_workspace};
use crate::project::executables::add_executable;
use crate::project::resolve_build_dir;
use crate::toolchain::gcc::{compile_to_object_with_dependencies, invoke_gcc, is_gcc_available};
use rayon::ThreadPoolBuilder;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

fn needs_recompilation(
    target_path: PathBuf,
    target_file_o: PathBuf,
    target_file_d: PathBuf,
) -> bool {
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
    cflags: &[String],
) -> anyhow::Result<String> {
    // path to generated .o in the .pronto folder
    // A gcc failure here is a user code error (GccError), propagated as-is
    // so main() can render it without the GitHub footer.
    let object_file_path = compile_to_object_with_dependencies(target_path, build_path, cflags)?;
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
    cflags: &[String],
) -> anyhow::Result<Vec<String>> {
    let canonicalized_path = canonicalize(target_path.clone())?;

    if visited.contains(&canonicalized_path) {
        return Ok(Vec::new());
    }
    visited.insert(canonicalized_path.clone());
    let target = target_path.file_name().unwrap();

    // Path to .o in .pronto dir
    let target_path_in_build_dir = build_path.join(target_path.clone());
    let target_file_o = target_path_in_build_dir.with_extension("o");
    let target_file_d = target_path_in_build_dir.with_extension("d");

    let mut objects: Vec<String> = Vec::new();

    // if c file has been modified since .o has been created.
    // Note: any non-empty cflags force recompilation, since the cache does
    // not track which flags produced the existing .o files.
    if !cflags.is_empty()
        || needs_recompilation(
            target_path.clone(),
            target_file_o.clone(),
            target_file_d.clone(),
        )
    {
        let path_str = compile_single_object(
            target,
            target_path.clone(),
            build_path.to_path_buf(),
            cflags,
        )?;
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
                if let Dependency::Header {
                    file: _,
                    source_file: Some(source_file),
                } = dependency
                {
                    objects.append(&mut compile_object_recursively(
                        source_file,
                        build_path.clone(),
                        visited,
                        cflags,
                    )?);
                }
            }
        }
        Err(_) => {
            // TODO : Handle error
        }
    }

    Ok(objects)
}

pub fn compile_entry(target: String, cflags: &[String]) -> anyhow::Result<PathBuf> {
    if !target.ends_with(".c") {
        anyhow::bail!("Expected a C file as argument.");
    }

    if !is_gcc_available() {
        anyhow::bail!("gcc not available.");
    }

    let target_path = PathBuf::from(&target);
    let tmp_build_path = prepare_workspace(Path::new(&target))?;

    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let pool = ThreadPoolBuilder::new().num_threads(num_threads).build()?;

    // Shared state across workers.
    let visited: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
    let objects: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let first_error: Arc<Mutex<Option<anyhow::Error>>> = Arc::new(Mutex::new(None));

    pool.install(|| {
        rayon::scope(|s| {
            compile_and_discover(
                target_path.clone(),
                tmp_build_path.clone(),
                cflags,
                Arc::clone(&visited),
                Arc::clone(&objects),
                Arc::clone(&first_error),
                s,
            );
        });
    });

    // A worker failure aborts the build and discards the tmp workspace.
    if let Some(err) = first_error.lock().unwrap().take() {
        let _ = rollback_workspace(Path::new(&target));
        return Err(err);
    }

    let object_paths = std::mem::take(&mut *objects.lock().unwrap());
    let executable_path = target_path.with_extension("");

    // Link into the tmp workspace first, so a failed link never touches the
    // previous executable or the committed build directory.
    let executable_name = executable_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Could not determine executable name"))?
        .to_owned();

    let tmp_executable_path = tmp_build_path.join(&executable_name);

    let mut link_args = object_paths;
    link_args.extend(cflags.iter().cloned());
    link_args.push("-o".to_string());
    link_args.push(
        tmp_executable_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Could not convert path into string"))?
            .to_string(),
    );

    if let Err(err) = invoke_gcc(link_args) {
        let _ = rollback_workspace(Path::new(&target));
        return Err(err);
    }

    // Everything succeeded: publish the new build directory, then the binary.
    commit_workspace(Path::new(&target))?;

    let build_dir = resolve_build_dir(Some(Path::new(&target)));
    let committed_executable = build_dir.join(&executable_name);
    std::fs::rename(&committed_executable, &executable_path)
        .map_err(|e| anyhow::anyhow!("Failed to move executable: {}", e))?;

    add_executable(executable_path.clone(), Some(&target_path))?;
    println!("\nBuilt executable at path : {:?}", executable_path);

    Ok(executable_path)
}

fn compile_and_discover<'a>(
    file: PathBuf,
    build_path: PathBuf,
    cflags: &'a [String],
    visited: Arc<Mutex<HashSet<PathBuf>>>,
    objects: Arc<Mutex<Vec<String>>>,
    first_error: Arc<Mutex<Option<anyhow::Error>>>,
    s: &rayon::Scope<'a>,
) {
    if first_error.lock().unwrap().is_some() {
        return;
    }

    let dependencies = compile_if_needed_and_parse_deps(
        file,
        &build_path,
        cflags,
        Arc::clone(&visited),
        Arc::clone(&objects),
        Arc::clone(&first_error),
    );

    for dependency in dependencies {
        let build_path = build_path.clone();
        let visited = Arc::clone(&visited);
        let objects = Arc::clone(&objects);
        let first_error = Arc::clone(&first_error);
        s.spawn(move |s| {
            compile_and_discover(
                dependency,
                build_path,
                cflags,
                visited,
                objects,
                first_error,
                s,
            );
        });
    }
}

fn compile_if_needed_and_parse_deps(
    file: PathBuf,
    build_path: &Path,
    cflags: &[String],
    visited: Arc<Mutex<HashSet<PathBuf>>>,
    objects: Arc<Mutex<Vec<String>>>,
    first_error: Arc<Mutex<Option<anyhow::Error>>>,
) -> Vec<PathBuf> {
    if first_error.lock().unwrap().is_some() {
        return Vec::new();
    }

    // Deduplicate using the canonical path.
    let canonical = match canonicalize(&file) {
        Ok(path) => path,
        Err(err) => {
            record_error(&first_error, err.into());
            return Vec::new();
        }
    };

    if !visited.lock().unwrap().insert(canonical) {
        return Vec::new();
    }

    let target_path_in_build_dir = build_path.join(&file);
    let target_file_o = target_path_in_build_dir.with_extension("o");
    let target_file_d = target_path_in_build_dir.with_extension("d");

    let should_compile = !cflags.is_empty()
        || needs_recompilation(file.clone(), target_file_o.clone(), target_file_d.clone());

    if should_compile {
        match compile_to_object_with_dependencies(file.clone(), build_path.to_path_buf(), cflags) {
            Ok(object_path) => {
                if let Some(path_str) = object_path.to_str() {
                    objects.lock().unwrap().push(path_str.to_string());
                }
                println!("Compiling {}...", file.display());
            }
            Err(err) => {
                record_error(&first_error, err);
                return Vec::new();
            }
        }
    } else {
        if let Some(path_str) = target_file_o.to_str() {
            objects.lock().unwrap().push(path_str.to_string());
        }
        println!("{} has not changed, no need to recompile.", file.display());
    }

    match parse_dot_d_dependencies(target_file_d) {
        Ok(dependencies) => dependencies
            .into_iter()
            .filter_map(|dependency| match dependency {
                Dependency::Header {
                    source_file: Some(source_file),
                    ..
                } => Some(source_file),
                _ => None,
            })
            .collect(),
        Err(_) => Vec::new(), // TODO: handle corrupt .d file
    }
}

fn record_error(slot: &Arc<Mutex<Option<anyhow::Error>>>, err: anyhow::Error) {
    let mut slot = slot.lock().unwrap();
    if slot.is_none() {
        *slot = Some(err);
    }
}
