use clap::{CommandFactory, Parser, Subcommand};

#[derive(Debug, PartialEq)]
pub enum CliContext {
    Init,
    Compile {
        target: String,
        cflags: Vec<String>,
    },
    Run {
        target: Option<String>,
        program_args: Vec<String>,
        cflags: Vec<String>,
    },
    Clean,
    FullClean,
    Version,
    Update,
}

#[derive(Parser, Debug)]
#[command(
    name = "pronto",
    about = "A lightning-fast, zero-config build system for C projects.",
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Shortcut for `pronto compile <FILE>`: `pronto <file.c>`
    #[arg(value_name = "FILE")]
    file: Option<String>,

    /// Print the compiled Pronto version details
    #[arg(short = 'v', long = "version", global = true)]
    version_flag: bool,

    /// Extra flags forwarded to gcc for compile and link steps.
    /// Example: `pronto --cflags "-Wall -O2" main.c`.
    /// Falls back to the `CFLAGS` environment variable when omitted.
    #[arg(
        long = "cflags",
        value_name = "FLAGS",
        global = true,
        allow_hyphen_values = true
    )]
    cflags: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a pronto project in current directory
    Init,
    /// Compile a specific C file and its dependencies
    #[command(alias = "build")]
    Compile {
        /// C file to compile
        target: String,
    },
    /// Compile, execute, and forward args after `--` to the program
    Run {
        /// C file or directory to run (defaults to ./main.c or ./src/main.c)
        target: Option<String>,
        /// Arguments forwarded to the program
        #[arg(last = true)]
        program_args: Vec<String>,
    },
    /// Remove the produced executables (use --full to also remove .pronto)
    Clean {
        /// Also remove the .pronto directory
        #[arg(long)]
        full: bool,
    },
    /// Remove executables and the .pronto directory (same as `clean --full`)
    FullClean,
    /// Download and install the latest version via the official script
    Update,
    /// Print the compiled Pronto version details
    Version,
}

/// Split a flags string the way a shell would: whitespace-separated,
/// honoring single and double quotes (e.g. `-DNAME="foo bar"` stays one flag).
fn split_flags(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut in_token = false;

    for c in s.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else {
                    cur.push(c);
                }
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    in_token = true;
                }
                c if c.is_whitespace() => {
                    if in_token {
                        out.push(std::mem::take(&mut cur));
                        in_token = false;
                    }
                }
                _ => {
                    cur.push(c);
                    in_token = true;
                }
            },
        }
    }
    if in_token {
        out.push(cur);
    }
    out
}

/// Effective cflags: `--cflags` CLI flag wins, then the `CFLAGS` env var,
/// otherwise empty.
fn resolve_cflags(cli_cflags: Option<String>) -> Vec<String> {
    let raw = cli_cflags.or_else(|| std::env::var("CFLAGS").ok());
    raw.map(|s| split_flags(&s)).unwrap_or_default()
}

pub fn parse_args(args: Vec<String>) -> Result<CliContext, clap::Error> {
    let cli = Cli::try_parse_from(args)?;

    if cli.version_flag {
        return Ok(CliContext::Version);
    }

    if let Some(file) = cli.file {
        if cli.command.is_some() {
            let mut cmd = Cli::command();
            return Err(cmd.error(
                clap::error::ErrorKind::UnknownArgument,
                format!("Unexpected file argument `{}` with subcommand.", file),
            ));
        }
        if file.ends_with(".c") {
            return Ok(CliContext::Compile {
                target: file,
                cflags: resolve_cflags(cli.cflags),
            });
        }
        let mut cmd = Cli::command();
        return Err(cmd.error(
            clap::error::ErrorKind::InvalidValue,
            format!("Unknown command `{}`", file),
        ));
    }

    match cli.command {
        Some(Commands::Init) => Ok(CliContext::Init),
        Some(Commands::Compile { target }) => Ok(CliContext::Compile {
            target,
            cflags: resolve_cflags(cli.cflags),
        }),
        Some(Commands::Run {
            target,
            program_args,
        }) => Ok(CliContext::Run {
            target,
            program_args,
            cflags: resolve_cflags(cli.cflags),
        }),
        Some(Commands::Clean { full }) => {
            if full {
                Ok(CliContext::FullClean)
            } else {
                Ok(CliContext::Clean)
            }
        }
        Some(Commands::FullClean) => Ok(CliContext::FullClean),
        Some(Commands::Update) => Ok(CliContext::Update),
        Some(Commands::Version) => Ok(CliContext::Version),
        None => {
            let mut cmd = Cli::command();
            Err(cmd.error(
                clap::error::ErrorKind::MissingRequiredArgument,
                "Missing command. Please run `pronto --help` to know more.",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    /// Serializes tests that read/mutate the `CFLAGS` env var (cargo runs
    /// tests in parallel threads of one process).
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn without_cflags_env() -> std::sync::MutexGuard<'static, ()> {
        let guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("CFLAGS") };
        guard
    }

    #[test]
    fn shortcut_c_file_compiles() {
        // Ensure no CFLAGS leaks in from the environment for deterministic tests.
        let _guard = without_cflags_env();
        assert_eq!(
            parse_args(args(&["pronto", "main.c"])).unwrap(),
            CliContext::Compile {
                target: "main.c".to_string(),
                cflags: vec![],
            }
        );
    }

    #[test]
    fn explicit_compile_and_build_alias() {
        let _guard = without_cflags_env();
        assert_eq!(
            parse_args(args(&["pronto", "compile", "src/main.c"])).unwrap(),
            CliContext::Compile {
                target: "src/main.c".to_string(),
                cflags: vec![],
            }
        );
        assert_eq!(
            parse_args(args(&["pronto", "build", "src/main.c"])).unwrap(),
            CliContext::Compile {
                target: "src/main.c".to_string(),
                cflags: vec![],
            }
        );
    }

    #[test]
    fn run_without_target() {
        let _guard = without_cflags_env();
        assert_eq!(
            parse_args(args(&["pronto", "run"])).unwrap(),
            CliContext::Run {
                target: None,
                program_args: vec![],
                cflags: vec![],
            }
        );
    }

    #[test]
    fn run_with_target_and_program_args() {
        let _guard = without_cflags_env();
        assert_eq!(
            parse_args(args(&["pronto", "run", "main.c", "--", "foo", "bar"])).unwrap(),
            CliContext::Run {
                target: Some("main.c".to_string()),
                program_args: vec!["foo".to_string(), "bar".to_string()],
                cflags: vec![],
            }
        );
    }

    #[test]
    fn clean_full_flag_maps_to_full_clean() {
        assert_eq!(
            parse_args(args(&["pronto", "clean", "--full"])).unwrap(),
            CliContext::FullClean
        );
        assert_eq!(
            parse_args(args(&["pronto", "clean"])).unwrap(),
            CliContext::Clean
        );
        assert_eq!(
            parse_args(args(&["pronto", "full-clean"])).unwrap(),
            CliContext::FullClean
        );
    }

    #[test]
    fn version_flags() {
        assert_eq!(
            parse_args(args(&["pronto", "-v"])).unwrap(),
            CliContext::Version
        );
        assert_eq!(
            parse_args(args(&["pronto", "--version"])).unwrap(),
            CliContext::Version
        );
        assert_eq!(
            parse_args(args(&["pronto", "version"])).unwrap(),
            CliContext::Version
        );
    }

    #[test]
    fn missing_command_errors() {
        assert!(parse_args(args(&["pronto"])).is_err());
    }

    #[test]
    fn unknown_command_errors() {
        assert!(parse_args(args(&["pronto", "frobnicate"])).is_err());
    }

    #[test]
    fn help_exits_via_clap_error() {
        let err = parse_args(args(&["pronto", "--help"])).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn split_flags_handles_quotes() {
        assert_eq!(
            split_flags("-Wall -O2 -DNAME=\"foo bar\" -DOTHER='a b'"),
            vec!["-Wall", "-O2", "-DNAME=foo bar", "-DOTHER=a b"]
        );
        assert!(split_flags("").is_empty());
        assert!(split_flags("   ").is_empty());
    }

    #[test]
    fn cflags_cli_flag_after_subcommand() {
        // No env access: `--cflags` wins, env is never read.
        assert_eq!(
            parse_args(args(&[
                "pronto",
                "compile",
                "main.c",
                "--cflags",
                "-Wall -O2"
            ]))
            .unwrap(),
            CliContext::Compile {
                target: "main.c".to_string(),
                cflags: vec!["-Wall".to_string(), "-O2".to_string()],
            }
        );
    }

    #[test]
    fn cflags_global_flag_with_shortcut() {
        assert_eq!(
            parse_args(args(&["pronto", "--cflags", "-O3", "main.c"])).unwrap(),
            CliContext::Compile {
                target: "main.c".to_string(),
                cflags: vec!["-O3".to_string()],
            }
        );
    }

    #[test]
    fn cflags_run_forwards_to_build() {
        assert_eq!(
            parse_args(args(&[
                "pronto", "run", "main.c", "--cflags", "-Wall", "--", "foo"
            ]))
            .unwrap(),
            CliContext::Run {
                target: Some("main.c".to_string()),
                program_args: vec!["foo".to_string()],
                cflags: vec!["-Wall".to_string()],
            }
        );
    }

    #[test]
    fn cflags_env_fallback() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("CFLAGS", "-O2 -Wall") };
        let ctx = parse_args(args(&["pronto", "compile", "main.c"])).unwrap();
        unsafe { std::env::remove_var("CFLAGS") };
        assert_eq!(
            ctx,
            CliContext::Compile {
                target: "main.c".to_string(),
                cflags: vec!["-O2".to_string(), "-Wall".to_string()],
            }
        );
    }

    #[test]
    fn cflags_cli_wins_over_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("CFLAGS", "-O0") };
        let ctx = parse_args(args(&["pronto", "compile", "main.c", "--cflags", "-O3"])).unwrap();
        unsafe { std::env::remove_var("CFLAGS") };
        assert_eq!(
            ctx,
            CliContext::Compile {
                target: "main.c".to_string(),
                cflags: vec!["-O3".to_string()],
            }
        );
    }
}
