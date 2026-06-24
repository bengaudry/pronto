use std::process::Command;

pub fn check_gcc_installation() -> bool {
    let check_cmd_status = Command::new("gcc")
        .arg("--version")
        .status()
        .expect("gcc command not found");

    return check_cmd_status.success();
}
