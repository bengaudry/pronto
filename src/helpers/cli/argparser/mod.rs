use std::{io::{Error, ErrorKind}};

pub enum CliContext {
    Compile {
        target: String,
    },
    Run {
        target: String,
    },
    Clean,
    Version,
    Help,
    Update
}

pub fn parse_args(args: Vec<String>) -> Result<CliContext, Error> {
    if args.len() == 1 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "At least one arg is requred (a path, or run, clean...). Please run {} help to know more.",
                args[0]
            ),
        ));
    }

    let context: CliContext;

    if args.len() == 2 {
        let first_arg = args[1].clone();

        if first_arg.ends_with(".c") {
            context = CliContext::Compile {
                target: first_arg,
            };
        } else if first_arg == "run" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "run expects at least one argument (the path of the c file to compile).",
            ));
        } else if first_arg == "-h" || first_arg == "--help" {
            context = CliContext::Help;
        } else if first_arg == "-v" || first_arg == "--version" {
            context = CliContext::Version;
        } else if first_arg == "clean" {
            context = CliContext::Clean;
        } else if first_arg == "update" {
            context = CliContext::Update;
        } else {
            return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Unknown command `{}`", first_arg),
            ));
        }
    } else if args.len() == 3 && args[1] == "run" {
        context = CliContext::Run {
            target: args[2].clone(),
        };
    } else {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid argument provided.",
        ));
    }

    Ok(context)
}
