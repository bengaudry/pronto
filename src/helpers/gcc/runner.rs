use std::path::{PathBuf};
use std::process::{Command};
use std::io::{Error, ErrorKind};

pub fn run_gcc_cmd(args: Vec<String>) -> Result<(), Error> {
    let output = Command::new("gcc")
        .args(args)
        .output()?; 

    if !output.status.success() {
        let code = output.status.code().unwrap_or(101);
        
        let stderr_msg = String::from_utf8_lossy(&output.stderr);
        
        return Err(Error::new(
            ErrorKind::Other,
            format!("GCC a échoué avec le code {}.\nDétails :\n{}", code, stderr_msg)
        ));
    }
    
    Ok(())
}


pub fn generate_dot_o_and_dot_d(target_path: PathBuf, build_path: PathBuf) -> Result<PathBuf, Error> {
    // Create the path to the mirrored target in the .pronto dir
    let target_path_in_build_dir = build_path.join(&target_path);
    let target_file_o = target_path_in_build_dir.with_extension("o");
    let parent_dir = target_file_o.parent().expect("Could not get parent dir");
    std::fs::create_dir_all(parent_dir).expect("Could not create subdirectories");

    return run_gcc_cmd(Vec::from([
        "-MMD".to_string(),
        target_path.to_str().expect("").to_string(),
        "-c".to_string(),
        "-o".to_string(),
        target_file_o.to_str().expect("").to_string(),
    ])).and(Ok(target_file_o));
}
