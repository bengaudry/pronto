use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn get_executables_list_file_path() -> PathBuf {
    Path::new(".pronto/executables.txt").to_path_buf()
}

pub fn parse_executables() -> Vec<PathBuf> {
    let fp = get_executables_list_file_path();
    if !fp.exists() || !fp.is_file() {
        return Vec::new();
    }

    let file = File::open(fp).expect("Could not open executables.txt");

    let mut executables_list: Vec<PathBuf> = Vec::new();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        executables_list.push(Path::new(line.trim()).to_path_buf());
    }

    executables_list
}

pub fn add_executable(executable_path: PathBuf) {
    let mut executables_list = parse_executables();

    if !executable_path.is_file() {
        panic!("Could not find executable file at {}", executable_path.display());
    }

    if !executables_list.contains(&executable_path) {
        executables_list.push(executable_path);
        save_executables(executables_list);
    }
}

pub fn remove_executable(executable_path: PathBuf) {
    let mut executables_list = parse_executables();
    executables_list.retain(|p| p != &executable_path);
    save_executables(executables_list);
}

pub fn save_executables(executables_list: Vec<PathBuf>) {
    let fp = get_executables_list_file_path();
    if fp.exists() && !fp.is_file() {
        panic!(".pronto/executables.txt already exists, and is not a file");
    }

    let file = File::create(fp).expect("Could not open executables.txt");
    let mut writer = BufWriter::new(file);

    for exe in executables_list {
        if let Some(exe_str) = exe.to_str() {
            writeln!(writer, "{}", exe_str).expect("Failed to write executable path to file");
        }
    }
}
