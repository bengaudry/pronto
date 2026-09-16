use crate::{clean_executables, compile};
use crate::helpers::cli::build_dir::{PRONTO_DIR, create_build_dir_in_curr_dir_if_not_exists};
use crate::helpers::cli::gitignore::add_file_to_local_gitignore;
use crate::version::{get_pronto_version, has_update_available, update_pronto};
use std::path::Path;
use std::process::Command;
use crate::cli::ui::HELP_TEXT;

pub fn handle_init() -> anyhow::Result<()> {
    println!("Initializing pronto project in current directory.");
    if create_build_dir_in_curr_dir_if_not_exists().is_err() {
        println!("A pronto project already exists here. Ending initialization.");
    } else {
        add_file_to_local_gitignore(Path::new(PRONTO_DIR).to_path_buf()).unwrap_or_else(|_| {
            println!("Could not add .pronto dir to gitignore file.");
        });
    }

    Ok(())
}

pub fn handle_compile(target: String) -> anyhow::Result<()> {
    compile(target)?;
    Ok(())
}

pub fn handle_run(target: String) -> anyhow::Result<()> {
    let executable_path = compile(target)?;
    println!("\n===== PROGRAM OUTPUT =====\n");
    let output = Command::new(format!("./{}", executable_path.to_str().unwrap()))
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run program: {}", e))?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Program failed :\n{}", stderr);
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
    if !has_update_available() {
        println!("Pronto already up to date.");
        return Ok(());
    }
    update_pronto();
    Ok(())
}

pub fn handle_help() -> anyhow::Result<()> {
    print!("{}", HELP_TEXT);
    Ok(())
}
