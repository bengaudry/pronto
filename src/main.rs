use shadow_rs::shadow;
shadow!(build);

mod helpers;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, panic};

use crate::helpers::cli::argparser::{CliContext, parse_args};
use crate::helpers::cli::build_dir::create_build_dir_if_not_exists;
use crate::helpers::cli::timestamps::is_file_newer;
use crate::helpers::gcc::check_installation::check_gcc_installation;
use crate::helpers::gcc::dependencies::Dependency;
use crate::helpers::gcc::dependencies::analyzer::analyse_dot_d_file;
use crate::helpers::gcc::runner::{generate_dot_o_and_dot_d, run_gcc_cmd};

const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn setup_panic_messages() {
    panic::set_hook(Box::new(|panic_info| {
        println!("\n{}{}🛑 [Pronto Error]{}\n", RED, BOLD, RESET);

        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("{}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            println!("{}", s);
        }

        println!(
            "\nIf this persists, please open an issue on GitHub (https://github.com/bengaudry/pronto/issues/new).\n"
        );
    }));
}

fn compile_obj(
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
        match is_file_newer(target_path.to_path_buf(), target_file_o.clone()) {
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
            generate_dot_o_and_dot_d(target_path.to_path_buf(), build_path.to_path_buf())
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
    match analyse_dot_d_file(target_file_d) {
        Ok(dependencies) => {
            // println!("Dependencies : ");
            for dependency in dependencies {
                // println!("{}", dependency);
                match dependency {
                    Dependency::Header { file, source_file } => {
                        if source_file.is_some() {
                            objects.append(&mut compile_obj(
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

fn compile(target: String) -> PathBuf {
    if !target.ends_with(".c") {
        panic!("Expected a C file as argument.")
    }

    if !check_gcc_installation() {
        panic!("gcc not available.")
    }

    // Create the .pronto dir at the cwd
    let build_path = create_build_dir_if_not_exists().unwrap_or_else(|err| {
        panic!("Could not create .pronto folder. Error : {}", err);
    });
    // println!("Build path {}\n", build_path.display());

    // Convert arg into Path
    let target_path = Path::new(&target);
    // println!("Target path : {}\n", target_path.display());

    let mut objects = compile_obj(target_path.to_path_buf(), build_path, &mut HashSet::new());

    // Build final executable
    let executable_path = target_path.with_extension("");
    objects.push("-o".to_string());
    objects.push(
        executable_path
            .to_str()
            .expect("Could not convert path into string")
            .to_string(),
    );
    run_gcc_cmd(objects).expect("Could not build executable");
    println!("\nBuilt executable at path : {:?}", executable_path);

    return executable_path;
}

fn update() {
    let curl_proc = Command::new("curl")
            .args(["-sSfL", "https://raw.githubusercontent.com/bengaudry/pronto/refs/heads/master/scripts/install.sh"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn curl");

    // 2. Start the sh process and set its stdin to capture curl's stdout
    let sh_proc = Command::new("sh")
        .stdin(curl_proc.stdout.unwrap()) // Take ownership of curl's stdout pipe
        .output()
        .expect("Failed to execute sh");

    // 3. Print out result
    if sh_proc.status.success() {
        println!("Installation complete!");
        println!("{}", String::from_utf8_lossy(&sh_proc.stdout));
    } else {
        eprintln!(
            "Installation failed:\n{}",
            String::from_utf8_lossy(&sh_proc.stderr)
        );
    }
}

fn main() {
    setup_panic_messages();

    let args: Vec<String> = env::args().collect();
    let cli_context = parse_args(args).expect("");

    match cli_context {
        CliContext::Compile { target } => {
            compile(target);
        }
        CliContext::Run { target } => {
            let executable_path = compile(target);
            println!("\n===== PROGRAM OUTPUT =====\n");
            let output = Command::new(format!("./{}", executable_path.to_str().unwrap()))
                .output()
                .expect("Failed to run program");

            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Program failed :\n{}", stderr);
            }
        }
        CliContext::Version => {
            println!("Pronto version: {}", build::TAG);
        }
        CliContext::Clean => { /* TODO */ }
        CliContext::Help => { /* TODO */ }
        CliContext::Update => update(),
    }
}
