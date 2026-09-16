use std::{io::{Error, ErrorKind}};

pub enum CliContext {
    Init,
    Compile {
        target: String,
    },
    Run {
        target: Option<String>,
        program_args: Vec<String>,
    },
    Clean,
    FullClean,
    Version,
    Help,
    Update
}

pub fn parse_args(args: Vec<String>) -> Result<CliContext, Error> {
    // Split at `--`: everything after is forwarded to the program (only `run` supports it).
    // e.g. `pronto run main.c -- foo bar` -> cli part `pronto run main.c`, program args `foo bar`.
    let (cli_args, program_args): (Vec<String>, Vec<String>) = match args.iter().position(|a| a == "--") {
        Some(pos) => (args[..pos].to_vec(), args[pos + 1..].to_vec()),
        None => (args, Vec::new()),
    };

    if cli_args.len() == 1 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "Missing command. Please run {} --help to know more.",
                cli_args[0]
            ),
        ));
    }

    // Only `run` can forward program arguments.
    let has_program_args = !program_args.is_empty();

    let context: CliContext;

    if cli_args.len() == 2 {
        let first_arg = cli_args[1].clone();

        if first_arg.ends_with(".c") {
            if has_program_args {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Only `pronto run` supports program arguments after `--`.",
                ));
            }
            context = CliContext::Compile {
                target: first_arg,
            };
        } else if first_arg == "init" {
            if has_program_args {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Only `pronto run` supports program arguments after `--`.",
                ));
            }
            context = CliContext::Init;
        } else if first_arg == "run" {
            context = CliContext::Run { target: None, program_args };
        } else if first_arg == "-h" || first_arg == "--help" {
            if has_program_args {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Only `pronto run` supports program arguments after `--`.",
                ));
            }
            context = CliContext::Help;
        } else if first_arg == "-v" || first_arg == "--version" {
            if has_program_args {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Only `pronto run` supports program arguments after `--`.",
                ));
            }
            context = CliContext::Version;
        } else if first_arg == "clean" {
            if has_program_args {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Only `pronto run` supports program arguments after `--`.",
                ));
            }
            context = CliContext::Clean;
        } else if first_arg == "full-clean" {
            if has_program_args {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Only `pronto run` supports program arguments after `--`.",
                ));
            }
            context = CliContext::FullClean;
        } else if first_arg == "update" {
            if has_program_args {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Only `pronto run` supports program arguments after `--`.",
                ));
            }
            context = CliContext::Update;
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Unknown command `{}`", first_arg),
            ));
        }
    } else if cli_args.len() == 3 && cli_args[1] == "run" {
        context = CliContext::Run {
            target: Some(cli_args[2].clone()),
            program_args,
        };
    } else {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid argument provided.",
        ));
    }

    Ok(context)
}
