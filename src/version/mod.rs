use std::process::{Command, Stdio};
use crate::build;

pub fn get_latest_version() -> Option<String> {
    // We fetch the latest release page and configure curl to follow redirects (-L)
    // and only output the final redirected URL (-w) while silencing the output (-o)
    let output = Command::new("curl")
        .args([
            "-sI",
            "-L",
            "-w",
            "%{url_effective}",
            "-o",
            "/dev/null",
            "https://github.com/bengaudry/pronto/releases/latest",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let url = String::from_utf8_lossy(&out.stdout);
            // The URL looks like: https://github.com/bengaudry/pronto/releases/tag/v1.1.0
            // We split by the last slash to isolate the tag "v1.1.0"
            if let Some(tag) = url.trim().split('/').last() {
                return Some(tag.to_string());
            }
        }
    }
    None
}

pub fn update_pronto() {
    let curl_proc = Command::new("curl")
        .args(["-sSfL", "https://raw.githubusercontent.com/bengaudry/pronto/refs/heads/master/scripts/install.sh"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn curl");

    // 2. Start the sh process and set its stdin to capture curl's stdout
    let sh_proc = Command::new("sh")
        .stdin(curl_proc.stdout.unwrap()) // Take ownership of curl's stdout pipe
        .output()
        .expect("Failed to execute sh");

    // 3. Print out result
    if sh_proc.status.success() {
        println!("Installation complete!");
        println!("{}", String::from_utf8_lossy(&sh_proc.stdout));
    } else {
        eprintln!(
            "Installation failed:\n{}",
            String::from_utf8_lossy(&sh_proc.stderr)
        );
    }
}

pub fn get_pronto_version() -> String {
    build::TAG.to_string()
}

pub fn has_update_available() -> bool {
    let current_pronto_version = get_pronto_version();
    let latest_pronto_version = get_latest_version();
    latest_pronto_version.is_some()
        && latest_pronto_version.clone().unwrap() != current_pronto_version
}
