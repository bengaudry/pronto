pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

pub const HELP_TEXT: &str = concat!(
"\x1b[1m\x1b[32mPronto\x1b[0m - A lightning-fast, zero-config build system for C projects.\n\n",
"\x1b[1mUSAGE:\x1b[0m\n",
"    pronto <filename.c>       Compile a specific C file and its dependencies\n",
"    pronto run [target] [-- args]  Compile and run (defaults to ./main.c or ./src/main.c)\n",
"    pronto [COMMAND]\n\n",
"\x1b[1mCOMMANDS:\x1b[0m\n",
"    run [target] [-- args]    Compile, execute, and forward args after `--` to the program\n",
"    clean                     Remove the produced executables (use --full to also remove .pronto)\n",
"    update                    Download and install the latest version via the official script\n",
"    help, -h, --help          Print this help infrastructure information\n",
"    -v, --version             Print the compiled Pronto version details\n\n",
"\x1b[1mEXAMPLES:\x1b[0m\n",
"    pronto src/main.c\n",
"    pronto run main.c\n",
"    pronto run main.c -- foo bar\n",
"    pronto run -- --help\n"
);
