# Pronto compilation process

Pronto builds a C project in a single pass using a work queue. It discovers source files as it goes, compiles independent files in parallel, and links everything at the end. Builds are transactional: `.pronto/build` is only replaced once every step succeeds.

## Flow

```text
  .pronto/build exists? ─────┐
       │                     │
       │ yes                 │ no
       │                     │
       v                     v
  copy contents      start with empty
  into .pronto/tmp   .pronto/tmp
       │                     │
       └──────────┬──────────┘
                  │
                  v
         +------------------+
         |   Entry file.c   |
         +---------+--------+
                   |
                   v
         +------------------+
         |   Work queue     |
         +---------+--------+
                   |
       +-----------+-----------+
       |           |           |
       v           v           v
+---------------+ +---------------+ +---------------+
| Worker thread | | Worker thread | | Worker thread |
+-------+-------+ +-------+-------+ +-------+-------+
        |                 |                 |
        v                 v                 v
+------------------------------------------------+
|  For each source file pulled from the queue:   |
|                                                |
|  1. Skip if already visited                    |
|  2. Check if .o is older than source/headers   |
|  3. Compile only if needed                     |
|  4. Parse the generated .d file                |
|  5. Push any new .c files to the queue         |
+------------------------------------------------+
             |
             v
   +------------------+
   |   Link objects   |
   +---------+--------+
             |
     +-------+-------+
     |               |
 success          failure
     |               |
     v               v
swap tmp→build   delete tmp
move executable  keep old build
next to source   and executable
```

## Explanation

1. **Prepare a temporary workspace** — if `.pronto/build` exists, copy its contents into `.pronto/tmp`. If it does not exist, start with an empty `.pronto/tmp`. This preserves incremental build artifacts when available while keeping `.pronto/build` untouched.
2. **Start with the entry file** — the `.c` file the user asked to build.
3. **Use a shared work queue** — worker threads take files from it one at a time.
4. **Compile only what changed** — if the object file is up to date, skip compilation.
5. **Discover dependencies** — after compiling (or using a cached `.d` file), find headers and their matching `.c` implementations.
6. **Add new sources to the queue** — this keeps the graph exploration going without a separate discovery phase.
7. **Link everything** — once the queue is empty, link all object files into a temporary executable inside `.pronto/tmp`.
8. **Commit or rollback** —
   - On success: rename `.pronto/tmp` to `.pronto/build`, then move the executable next to the source file.
   - On failure: delete `.pronto/tmp` and leave the previous build and executable untouched.

This avoids scanning the project twice, keeps incremental builds fast, compiles independent `.c` files in parallel, and ensures `.pronto/build` and the generated executable are never left in a partially-built state.
