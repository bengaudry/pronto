use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub mod analyzer;

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
            Dependency::Header { file, source_file } => {
                match source_file {
                    Some(src) => write!(
                        f,
                        "[Header] {} (Implementation: {})",
                        file.display(),
                        src.display()
                    ),
                    None => write!(
                        f,
                        "[Header] {} (Implementation: None)",
                        file.display()
                    ),
                }
            }
            Dependency::Source { file } => {
                write!(f, "[Source] {}", file.display())
            }
        }
    }
}
