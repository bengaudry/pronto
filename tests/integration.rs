use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Base directory for integration test fixtures.
const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

/// Returns the path to the compiled `pronto` binary. Cargo makes this
/// environment variable available automatically for integration tests.
fn pronto_bin() -> &'static str {
    env!("CARGO_BIN_EXE_pronto")
}

/// Copy a fixture directory to an isolated temporary location outside the
/// repository tree. This is important because Pronto resolves the project root
/// by walking up the filesystem looking for `.git` or `.pronto`; running
/// inside the repo would place `.pronto` at the repository root.
fn copy_fixture(name: &str) -> tempfile::TempDir {
    let src = Path::new(FIXTURES_DIR).join(name);
    assert!(
        src.is_dir(),
        "fixture {} not found at {}",
        name,
        src.display()
    );

    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    copy_dir_recursively(&src, tmp.path()).unwrap_or_else(|e| {
        panic!(
            "failed to copy fixture {} to {}: {}",
            name,
            tmp.path().display(),
            e
        )
    });
    tmp
}

fn copy_dir_recursively(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursively(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// Run the `pronto` binary in the given working directory with the supplied
/// arguments and capture its output.
fn run_pronto(dir: &Path, args: &[&str]) -> Output {
    Command::new(pronto_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "failed to execute pronto {:?} in {}: {}",
                args,
                dir.display(),
                e
            )
        })
}

/// Run the compiled executable at `relative_path` inside `dir` and capture its
/// output. `args` are forwarded to the program. If `input` is provided it is
/// written to the child's stdin.
fn run_executable(dir: &Path, relative_path: &str, args: &[&str], input: Option<&str>) -> Output {
    let exe = dir.join(relative_path);
    let mut cmd = Command::new(&exe);
    cmd.current_dir(dir).args(args);

    if input.is_some() {
        cmd.stdin(Stdio::piped());
    }

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            panic!(
                "failed to spawn executable {} in {}: {}",
                exe.display(),
                dir.display(),
                e
            )
        });

    if let Some(data) = input {
        child
            .stdin
            .take()
            .expect("stdin was requested but is not available")
            .write_all(data.as_bytes())
            .expect("failed to write to child stdin");
    }

    child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("failed to wait for executable {}: {}", exe.display(), e))
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        context,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_stdout_contains(output: &Output, needle: &str, context: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(needle),
        "{}: expected stdout to contain {:?}, got:\n{}",
        context,
        needle,
        stdout
    );
}

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{} expected to fail but succeeded\nstdout:\n{}",
        context,
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn hello_world_compiles_and_runs() {
    let dir = copy_fixture("1-hello-world");
    let out = run_pronto(dir.path(), &["main.c"]);
    assert_success(&out, "pronto main.c");

    let exe = run_executable(dir.path(), "main", &[], None);
    assert_success(&exe, "run hello-world");
    assert_stdout_contains(&exe, "Hello, world!\n", "hello-world output");
}

#[test]
fn header_only_compiles_and_runs() {
    let dir = copy_fixture("2-header-only");
    let out = run_pronto(dir.path(), &["compile", "main.c"]);
    assert_success(&out, "pronto compile main.c");

    let exe = run_executable(dir.path(), "main", &[], None);
    assert_success(&exe, "run header-only");
    assert_stdout_contains(&exe, "max: 10\n", "header-only max output");
    assert_stdout_contains(
        &exe,
        "Hello from inline header!\n",
        "header-only inline output",
    );
}

#[test]
fn header_include_compiles_and_runs() {
    let dir = copy_fixture("3-header-include");
    let out = run_pronto(dir.path(), &["build", "main.c"]);
    assert_success(&out, "pronto build main.c");

    let exe = run_executable(dir.path(), "main", &[], None);
    assert_success(&exe, "run header-include");
    assert_stdout_contains(&exe, "Hello World!\n", "header-include output");
}

#[test]
fn multi_file_compiles_and_runs() {
    let dir = copy_fixture("4-multi-file");
    let out = run_pronto(dir.path(), &["main.c"]);
    assert_success(&out, "pronto main.c (multi-file)");

    let exe = run_executable(dir.path(), "main", &[], None);
    assert_success(&exe, "run multi-file");
    assert_stdout_contains(&exe, "add: 8\n", "multi-file add output");
    assert_stdout_contains(&exe, "mul: 15\n", "multi-file mul output");
}

#[test]
fn nested_src_layout_compiles_and_runs() {
    let dir = copy_fixture("5-nested");
    let out = run_pronto(dir.path(), &["compile", "src/main.c"]);
    assert_success(&out, "pronto compile src/main.c");

    let exe = run_executable(dir.path(), "src/main", &[], None);
    assert_success(&exe, "run nested");
    assert_stdout_contains(&exe, "sum: 30\n", "nested sum output");
    assert_stdout_contains(&exe, "Hello, Pronto!\n", "nested greeting output");
}

#[test]
fn scanf_program_runs_with_piped_input() {
    let dir = copy_fixture("6-scanf");
    let out = run_pronto(dir.path(), &["main.c"]);
    assert_success(&out, "pronto main.c (scanf)");

    let exe = run_executable(dir.path(), "main", &[], Some("Alice\n30\n"));
    assert_success(&exe, "run scanf");
    assert_stdout_contains(&exe, "Hello Alice, you are 30\n", "scanf output");
}

#[test]
fn with_args_forwards_arguments() {
    let dir = copy_fixture("7-with-args");
    let out = run_pronto(dir.path(), &["main.c"]);
    assert_success(&out, "pronto main.c (with-args)");

    let no_args = run_executable(dir.path(), "main", &[], None);
    // Running without args exits with code 1.
    assert_failure(&no_args, "with-args without arguments should fail");
    assert_stdout_contains(&no_args, "Usage:", "with-args usage");

    let with_args = run_executable(dir.path(), "main", &["World"], None);
    assert_success(&with_args, "with-args with argument");
    assert_stdout_contains(&with_args, "Hello, World!\n", "with-args output");
    assert_stdout_contains(&with_args, "argv[1]=World", "with-args argv");
}

#[test]
fn transitive_dependencies_compiles_and_runs() {
    let dir = copy_fixture("8-transitive");
    let out = run_pronto(dir.path(), &["main.c"]);
    assert_success(&out, "pronto main.c (transitive)");

    let exe = run_executable(dir.path(), "main", &[], None);
    assert_success(&exe, "run transitive");
    assert_stdout_contains(&exe, "result a: 44\n", "transitive output");
}

#[test]
fn missing_header_fails_to_compile() {
    let dir = copy_fixture("9-error-missing-header");
    let out = run_pronto(dir.path(), &["main.c"]);
    assert_failure(&out, "pronto main.c (missing header)");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does_not_exist.h"),
        "expected stderr to mention missing header, got:\n{}",
        stderr
    );
}
