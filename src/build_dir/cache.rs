use std::fs::metadata;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

/// Returns true if the file1 timestamp is greater strictly than file2
pub fn is_source_newer_than_artifact(
    file1_path: PathBuf,
    file2_path: PathBuf,
) -> Result<bool, Error> {
    let file1_meta = metadata(&file1_path)
        .expect(&format!("Could not find file {:#?}.", file1_path).to_string());
    let file2_meta = metadata(&file2_path)
        .expect(&format!("Could not find file {:#?}.", file2_path).to_string());

    if !file1_meta.is_file() || !file2_meta.is_file() {
        return Err(Error::new(
            ErrorKind::IsADirectory,
            "Trying to compare dates on a directory.",
        ));
    }

    let file1_modif = file1_meta
        .modified()
        .expect("Time comparison not supported on this platform");
    let file2_modif = file2_meta
        .modified()
        .expect("Time comparison not supported on this platform");

    Ok(file1_modif > file2_modif)
}
