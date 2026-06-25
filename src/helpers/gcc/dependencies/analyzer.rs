use std::{fs::File, io::{BufRead, BufReader}, path::{PathBuf}};
use crate::helpers::gcc::dependencies::{Dependency};


fn find_source_file_for_header_file(header_file_path: PathBuf) -> Option<PathBuf> {
    let potential_source_file_path = header_file_path.with_extension("c");
    if potential_source_file_path.exists() {
        return Some(potential_source_file_path);
    }
    None
}

pub fn analyse_dot_d_file(dot_d_path: PathBuf) -> Result<Vec<Dependency>, String> {
    let file = File::open(dot_d_path).map_err(|err| format!("Could not open .d file. Error : {}", err))?;
    let reader = BufReader::new(file);

    let mut dependencies: Vec<Dependency> = Vec::new();

    let mut is_dep_newline = false;
    for line in reader.lines() {
        let line = line.map_err(|err| format!("Could not read line from .d file. Error : {}", err))?;
        let trimmed_line = line.trim();

        // Skip empty lines and lines that start with a comment
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        let mut tokens: Vec<String> = trimmed_line.split(": ").map(|s| s.to_string()).collect();
        if !is_dep_newline { // remove the object from the deps if analyzing new line
            tokens.remove(0);
        }

        let objs = tokens[0].split_whitespace().map(|s| s.to_string());

        for obj in objs {
            if obj == "\\" {
                is_dep_newline = true;
                break;
            }
            let dep: Dependency;

            let obj_path = PathBuf::from(obj.clone());
            if obj.ends_with(".c") {
                dep = Dependency::Source {
                    file: obj_path.clone()
                };
            } else if obj.ends_with(".h") {
                let source_file = find_source_file_for_header_file(obj_path.clone());
                dep = Dependency::Header {
                    file: obj_path.clone(),
                    source_file
                };
            } else {
                println!("{} file is not supported.", obj);
                continue;
            }
            dependencies.push(dep);
        }
    }

    Ok(dependencies)
}
