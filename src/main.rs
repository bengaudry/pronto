use std::path::Path;
use std::process::Command;
use std::{env, panic};

use pronto::clean_executables;
use pronto::compile;
use pronto::helpers::cli::argparser::{CliContext, parse_args};
use pronto::helpers::cli::build_dir::create_build_dir_in_curr_dir_if_not_exists;
use pronto::helpers::cli::build_dir::PRONTO_DIR;
use pronto::helpers::cli::gitignore::add_file_to_local_gitignore;
use pronto::versionning::{get_pronto_version, has_update_available, update_pronto};
use pronto::{BOLD, HELP_TEXT, RED, RESET, YELLOW};

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

fn try_main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let cli_context = parse_args(args).map_err(|e| anyhow::anyhow!("{}", e))?;

    match cli_context {
        CliContext::Init => {
            println!("Initializing pronto project in current directory.");
            if create_build_dir_in_curr_dir_if_not_exists().is_err() {
                println!("A pronto project already exists here. Ending initialization.");
                return Ok(());
            }
            add_file_to_local_gitignore(Path::new(PRONTO_DIR).to_path_buf()).unwrap_or_else(|_| {
                println!("Could not add .pronto dir to gitignore file.");
            });
        }
        CliContext::Compile { target } => {
            compile(target)?;
        }
        CliContext::Run { target } => {
            let executable_path = compile(target)?;
            println!("\n===== PROGRAM OUTPUT =====\n");
            let output = Command::new(format!("./{}", executable_path.to_str().unwrap()))
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to run program: {}", e))?;

            print!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprint!("Program failed :\n{}", stderr);
            }
        }
        CliContext::Version => {
            println!("Pronto version: {}", get_pronto_version());
        }
        CliContext::Clean => {
            clean_executables(false)?;
        }
        CliContext::FullClean => {
            clean_executables(true)?;
        }
        CliContext::Update => {
            if !has_update_available() {
                println!("Pronto already up to date.");
                return Ok(());
            }
            update_pronto()
        }
        CliContext::Help => {
            print!("{}", HELP_TEXT);
        }
    }

    Ok(())
}

fn main() {
    setup_panic_messages();

    if let Err(e) = try_main() {
        eprintln!("\n{}{}🛑 [Pronto Error]{}\n{:#}\n", RED, BOLD, RESET, e);
        eprintln!(
            "If this persists, please open an issue on GitHub (https://github.com/bengaudry/pronto/issues/new).\n"
        );
        std::process::exit(1);
    }

    if has_update_available() {
        println!(
            "{}A new version is available! Run {}`pronto update`{}{} to install it.{}",
            YELLOW, BOLD, RESET, YELLOW, RESET
        )
    }
}
