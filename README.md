# Pronto

A zero-config build system for C projects.

Pronto compiles your C programs with a single command, discovers header dependencies automatically, and only rebuilds what changed.

---

## Features

- **Zero configuration** — drop `pronto` in any C project and run it.
- **Automatic dependency discovery** — parses GCC `.d` files to rebuild headers and their source files when they change.
- **Incremental builds** — recompiles only what changed.
- **Simple CLI** — compile, run, clean, and update from one command.
- **Interactive program support** — `pronto run` forwards `stdin`/`stdout`/`stderr` directly to your program.
- **Custom flags** — pass extra GCC flags via `--cflags` or the `CFLAGS` environment variable.

---

## Installation

Install the latest release with the official installer:

```bash
curl -sSfL "https://raw.githubusercontent.com/bengaudry/pronto/refs/heads/master/scripts/install.sh" | sh
```

The binary is installed to `~/.local/bin`. Make sure that directory is in your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Build from source

Pronto is written in Rust. To build it locally:

```bash
# Clone the repository
git clone https://github.com/bengaudry/pronto.git
cd pronto

# Build the release binary
cargo build --release

# The binary is available at
target/release/pronto
```

---

## Quick start

Create a simple C file:

```c
// main.c
#include <stdio.h>

int main() {
    printf("Hello, world!\n");
    return 0;
}
```

Compile and run it:

```bash
# Compile a single file
pronto main.c

# Or compile and run in one step
pronto run main.c

# Run with arguments forwarded to your program
pronto run main.c -- arg1 arg2
```

---

## Usage

### Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `pronto <file.c>` | — | Shortcut to compile a C file. |
| `pronto init` | — | Initialize a Pronto project in the current directory (creates `.pronto/` and adds it to `.gitignore`). |
| `pronto compile <file.c>` | `pronto build <file.c>` | Compile a C file and its dependencies. |
| `pronto run [target]` | — | Compile, then execute the program. Forwards everything after `--` to the program. |
| `pronto clean` | — | Remove produced executables. |
| `pronto clean --full` | `pronto full-clean` | Remove executables **and** the `.pronto` directory. |
| `pronto version` | `pronto -v`, `pronto --version` | Print the installed Pronto version. |
| `pronto update` | — | Download and install the latest Pronto release. |

### Passing flags to GCC

Use `--cflags` to forward flags to GCC:

```bash
pronto main.c --cflags "-Wall -O2"
pronto run main.c --cflags "-std=c11 -Wextra"
```

You can also set the `CFLAGS` environment variable. The `--cflags` option takes precedence when both are set.

```bash
CFLAGS="-Wall -O2" pronto main.c
```

---

## How it works

1. Pronto resolves the project root by looking for `.git` or `.pronto` directories, or by using the current directory.
2. It creates a `.pronto/build` directory that mirrors your source tree.
3. It compiles each source file to an object file in `.pronto/build`, generating GCC dependency files (`.d`) to track `#include` relationships.
4. It links all required objects into an executable next to the target source file.
5. On subsequent runs, it skips files that have not changed since the last build.

---

## Project structure

A typical Pronto project looks like this:

```text
.
├── .pronto/            # Build cache and dependency metadata
├── main.c              # Entry point
├── utils.c             # Additional source files
├── utils.h
└── ...
```

You can also use a `src/` layout:

```text
.
├── .pronto/
├── src/
│   ├── main.c
│   └── ...
```

---

## Development

Run the test suite:

```bash
cargo test
```

Build the release binary:

```bash
cargo build --release
```

Run Clippy and formatting checks:

```bash
cargo clippy --all-targets
cargo fmt --check
```

---

## Requirements

- [GCC](https://gcc.gnu.org/) must be available in your `PATH`.
- (For building from source) [Rust](https://www.rust-lang.org/) 1.85 or later.

