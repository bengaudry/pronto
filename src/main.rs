mod helpers;

use std::env;
use std::fmt::format;
use std::path::Path;
use std::process::Command;

use crate::helpers::cli::argparser::{CliCommand, parse_args};
use crate::helpers::cli::build_dir::create_build_dir_if_not_exists;
use crate::helpers::cli::timestamps::is_file_newer;
use crate::helpers::gcc::check_installation::check_gcc_installation;
use crate::helpers::gcc::runner::{generate_dot_o_and_dot_d, run_gcc_cmd};

fn main() {
    let args: Vec<String> = env::args().collect();
    let cli_context = parse_args(args).expect("");

    if (cli_context.command == CliCommand::Run || cli_context.command == CliCommand::Compile)
        && cli_context.target != None
    {
        let target = cli_context.target.unwrap();

        if !target.ends_with(".c") {
            panic!("Expected a C file as argument.")
        }

        if !check_gcc_installation() {
            panic!("gcc not found.")
        }

        // Create the .pronto dir at the cwd
        let build_path = create_build_dir_if_not_exists().unwrap_or_else(|err| {
            panic!("Could not create .pronto folder. Error : {}", err);
        });
        println!("Build path {}\n", build_path.display());

        // Convert arg into Path
        let target_path = Path::new(&target);
        println!("Target path : {}\n", target_path.display());

        // Path to .o in .pronto dir
        let target_path_in_build_dir = build_path.join(&target_path);
        let target_file_o = target_path_in_build_dir.with_extension("o");
        let target_file_d = target_path_in_build_dir.with_extension("d");

        // Check if .o and .d already exists
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
            let object_file_path = generate_dot_o_and_dot_d(target_path.to_path_buf(), build_path)
                .expect("Could not use gcc for target.");
            let path_str = object_file_path
                .to_str()
                .expect("Invalid UTF-8")
                .to_string();
            objects.push(path_str);
            // TODO : recompile dependencies
        } else {
            objects.push(target_file_o.to_str().expect("Invalid UTF-8").to_string());
            println!("{} has not changed, no need to recompile.\n", target)
        }

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
        println!("Built executable at path : {:?}", executable_path);
        
        if cli_context.command == CliCommand::Run {
            println!("Running program...\n");
            let output = Command::new(format!("./{}", executable_path.to_str().unwrap()))
                .output()
                .expect("Failed to run program");

            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Program failed :\n{}", stderr);
            }
        }
    }

}
