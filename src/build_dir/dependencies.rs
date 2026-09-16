use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub enum Dependency {
    Header {
        file: PathBuf,
        source_file: Option<PathBuf>,
    },
    Source {
        file: PathBuf,
    },
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Dependency::Header { file, source_file } => match source_file {
                Some(src) => write!(
                    f,
                    "[Header] {} (Implementation: {})",
                    file.display(),
                    src.display()
                ),
                None => write!(f, "[Header] {} (Implementation: None)", file.display()),
            },
            Dependency::Source { file } => {
                write!(f, "[Source] {}", file.display())
            }
        }
    }
}

/// Finds the ".c" file associated with a ".h" if it exists, and returns the path to it (or None)
fn resolve_header_implementation(header_file_path: PathBuf) -> Option<PathBuf> {
    let potential_source_file_path = header_file_path.with_extension("c");
    if potential_source_file_path.exists() {
        return Some(potential_source_file_path);
    }
    None
}

pub fn parse_dot_d_dependencies(dot_d_path: PathBuf) -> Result<Vec<Dependency>, String> {
    let file =
        File::open(dot_d_path).map_err(|err| format!("Could not open .d file. Error : {}", err))?;
    let reader = BufReader::new(file);

    let mut dependencies: Vec<Dependency> = Vec::new();

    let mut is_dep_newline = false;
    for line in reader.lines() {
        let line =
            line.map_err(|err| format!("Could not read line from .d file. Error : {}", err))?;
        let trimmed_line = line.trim();

        // Skip empty lines and lines that start with a comment
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        let mut tokens: Vec<String> = trimmed_line.split(": ").map(|s| s.to_string()).collect();
        if !is_dep_newline {
            // remove the object from the deps if analyzing new line
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
                    file: obj_path.clone(),
                };
            } else if obj.ends_with(".h") {
                let source_file = resolve_header_implementation(obj_path.clone());
                dep = Dependency::Header {
                    file: obj_path.clone(),
                    source_file,
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
