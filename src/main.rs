use std::{env, panic};

use pronto::cli::args::{CliContext, parse_args};
use pronto::cli::commands;
use pronto::cli::ui::{BOLD, RED, RESET, YELLOW};
use pronto::version::has_update_available;

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
        CliContext::Init => commands::handle_init()?,
        CliContext::Compile { target } => commands::handle_compile(target)?,
        CliContext::Run { target } => commands::handle_run(target)?,
        CliContext::Version => commands::handle_version()?,
        CliContext::Clean => commands::handle_clean()?,
        CliContext::FullClean => commands::handle_full_clean()?,
        CliContext::Update => commands::handle_update()?,
        CliContext::Help => commands::handle_help()?,
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
