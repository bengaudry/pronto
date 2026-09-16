use std::{env, panic};

use pronto::cli::args::{CliContext, parse_args};
use pronto::cli::commands;
use pronto::cli::ui::{BOLD, RED, RESET, YELLOW};
use pronto::toolchain::gcc::GccError;
use pronto::version::is_update_available;

fn install_panic_hook() {
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

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let cli_context = match parse_args(args) {
        Ok(ctx) => ctx,
        // clap handles --help / parse errors itself with correct exit codes.
        Err(e) => e.exit(),
    };

    match cli_context {
        CliContext::Init => commands::handle_init()?,
        CliContext::Compile { target, cflags } => commands::handle_compile(target, cflags)?,
        CliContext::Run {
            target,
            program_args,
            cflags,
        } => commands::handle_run(target, program_args, cflags)?,
        CliContext::Version => commands::handle_version()?,
        CliContext::Clean => commands::handle_clean()?,
        CliContext::FullClean => commands::handle_full_clean()?,
        CliContext::Update => commands::handle_update()?,
    }

    Ok(())
}

fn main() {
    install_panic_hook();

    if let Err(e) = run() {
        // gcc failures are user code errors, not pronto bugs:
        // show gcc's stderr directly, without the GitHub footer.
        let gcc_err = e.chain().find_map(|c| c.downcast_ref::<GccError>());
        if let Some(gcc_err) = gcc_err {
            eprintln!("\n{}{}🛑 [GCC Error]{}\n{}\n", RED, BOLD, RESET, gcc_err);
            std::process::exit(1);
        }

        eprintln!("\n{}{}🛑 [Pronto Error]{}\n{:#}\n", RED, BOLD, RESET, e);
        eprintln!(
            "If this persists, please open an issue on GitHub (https://github.com/bengaudry/pronto/issues/new).\n"
        );
        std::process::exit(1);
    }

    if is_update_available() {
        println!(
            "{}A new version is available! Run {}`pronto update`{}{} to install it.{}",
            YELLOW, BOLD, RESET, YELLOW, RESET
        )
    }
}
