use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

const GITIGNORE_FILE_NAME: &str = ".gitignore";

pub fn create_gitignore_in_current_dir_if_not_exists() -> Result<PathBuf, Error> {
    let gitignore_path = Path::new(GITIGNORE_FILE_NAME);

    if !gitignore_path.is_file() {
        if gitignore_path.exists() {
            return Err(Error::new(
                ErrorKind::IsADirectory,
                ".gitignore is a directory",
            ));
        }
        File::create(gitignore_path).unwrap();
    }

    Ok(gitignore_path.to_path_buf())
}
pub fn add_file_to_local_gitignore(file_path: PathBuf) -> Result<(), Error> {
    let gitignore_path = create_gitignore_in_current_dir_if_not_exists()?;

    let mut gitignore_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&gitignore_path)?;

    let mut gitignore_file_contents = String::new();
    gitignore_file.read_to_string(&mut gitignore_file_contents)?;

    let file_name = file_path.to_str().expect("Chemin non UTF-8 valide");

    if !gitignore_file_contents.contains(file_name) {
        if !gitignore_file_contents.is_empty() {
            if !gitignore_file_contents.ends_with('\n') {
                gitignore_file_contents.push('\n');
            }
            gitignore_file_contents.push('\n');
        }

        if !gitignore_file_contents.contains("# Pronto") {
            gitignore_file_contents.push_str("# Pronto\n");
        }

        gitignore_file_contents.push_str(file_name);
        gitignore_file_contents.push_str("/\n");

        gitignore_file.set_len(0)?;
        use std::io::Seek;
        gitignore_file.seek(std::io::SeekFrom::Start(0))?;

        gitignore_file.write_all(gitignore_file_contents.as_bytes())?;
    }

    Ok(())
}
