use std::process::Command;

pub fn is_valgrind_available() -> bool {
    Command::new("valgrind")
        .arg("--version")
        .status()
        .is_ok_and(|status| status.success())
}
