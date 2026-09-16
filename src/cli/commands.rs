use crate::build_dir::{clean::clean_executables, linker::link_target};
use crate::project::gitignore::ensure_ignored_in_gitignore;
use crate::project::{PRONTO_DIR, ensure_build_dir};
use crate::version::{get_pronto_version, is_update_available, update_pronto};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn handle_init() -> anyhow::Result<()> {
    println!("Initializing pronto project in current directory.");
    if ensure_build_dir(None).is_err() {
        println!("A pronto project already exists here. Ending initialization.");
    } else {
        ensure_ignored_in_gitignore(Path::new(PRONTO_DIR).to_path_buf()).unwrap_or_else(|_| {
            println!("Could not add .pronto dir to gitignore file.");
        });
    }

    Ok(())
}

pub fn handle_compile(target: String, cflags: Vec<String>) -> anyhow::Result<()> {
    link_target(target, &cflags)?;
    Ok(())
}

fn resolve_run_target(target: Option<String>) -> anyhow::Result<String> {
    // Explicit file path: use it as-is (link_target will validate it).
    if let Some(t) = target {
        let p = Path::new(&t);
        if p.is_file() {
            return Ok(t);
        }
        if p.is_dir() {
            let candidate = p.join("main.c");
            if candidate.is_file() {
                return Ok(candidate.to_string_lossy().to_string());
            }
            // `pronto run .` in a src-layout project: try ./src/main.c
            if t == "." || t == "./" {
                let nested = p.join("src").join("main.c");
                if nested.is_file() {
                    return Ok(nested.to_string_lossy().to_string());
                }
            }
            anyhow::bail!(
                "No main.c found in '{}'. Please specify a file: pronto run <file.c>",
                t
            );
        }
        // Non-existent path: keep a clear error instead of delegating to the linker.
        if t.ends_with(".c") {
            anyhow::bail!(
                "File not found: '{}'. Please specify an existing .c file.",
                t
            );
        }
        anyhow::bail!(
            "'{}' is not a file or directory. Please specify a file: pronto run <file.c>",
            t
        );
    }

    // No argument: conventional locations.
    for candidate in ["main.c", "src/main.c"] {
        if Path::new(candidate).is_file() {
            return Ok(candidate.to_string());
        }
    }
    anyhow::bail!(
        "run expects at least one argument (the path of the c file to run). No main.c found in ./main.c or ./src/main.c."
    );
}

pub fn handle_run(
    target: Option<String>,
    program_args: Vec<String>,
    cflags: Vec<String>,
) -> anyhow::Result<()> {
    let resolved = resolve_run_target(target)?;
    let executable_path = link_target(resolved, &cflags)?;
    println!("\n===== PROGRAM OUTPUT =====\n");

    // Use inherit() so the child shares the terminal's stdin/stdout/stderr.
    // This is required for interactive programs (scanf, getchar, etc.):
    // - Command::output() pipes stdin as closed (EOF) and buffers output
    //   until the child exits, so scanf immediately gets EOF and prompts are invisible.
    // - inherit() forwards the TTY directly, no buffering, input works.
    // Program args after `--` are forwarded verbatim (e.g. `pronto run main.c -- foo`).
    let status = Command::new(format!("./{}", executable_path.display()))
        .args(program_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run program: {}", e))?;

    if !status.success() {
        if let Some(code) = status.code() {
            anyhow::bail!("Program exited with code {}", code);
        } else {
            anyhow::bail!("Program terminated by signal");
        }
    }

    Ok(())
}

pub fn handle_version() -> anyhow::Result<()> {
    println!("Pronto version: {}", get_pronto_version());
    Ok(())
}

pub fn handle_clean() -> anyhow::Result<()> {
    clean_executables(false)?;
    Ok(())
}

pub fn handle_full_clean() -> anyhow::Result<()> {
    clean_executables(true)?;
    Ok(())
}

pub fn handle_update() -> anyhow::Result<()> {
    if !is_update_available() {
        println!("Pronto already up to date.");
        return Ok(());
    }
    update_pronto();
    Ok(())
}
