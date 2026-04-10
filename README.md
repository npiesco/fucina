# fucina

Automated Rust repair pipeline: format → fix → verify → test.

Single cross-platform binary distributed via `cargo install` or pre-built releases.

## Install

### From source

```bash
cargo install --path .
```

### From crates.io (once published)

```bash
cargo install fucina
```

### Pre-built binaries

Download from [GitHub Releases](../../releases/latest) — builds are available for:

| Platform | Architecture | File |
|----------|-------------|------|
| Linux | x64 | `fucina-linux-x64.tar.gz` |
| Linux | ARM64 | `fucina-linux-arm64.tar.gz` |
| macOS | Intel | `fucina-macos-x64.tar.gz` |
| macOS | Apple Silicon | `fucina-macos-arm64.tar.gz` |
| Windows | x64 | `fucina-windows-x64.zip` |
| Windows | ARM64 | `fucina-windows-arm64.zip` |

Extract and add to your `PATH`.

## Usage

```bash
fucina              # full pipeline
fucina --no-test    # skip tests
fucina --dry-run    # show commands without executing
fucina --all-features
fucina --recursive  # run on all Rust projects under current dir
fucina -r -p ~/code # run on all Rust projects under ~/code
```

## Pipeline Steps

1. `cargo fmt --all` — format
2. `cargo clippy --fix` — auto-fix lint issues
3. `cargo clippy -- -D warnings` — verify clean
4. `cargo test` — run tests (unless `--no-test`)

In `--recursive` mode, fucina walks subdirectories to find all `Cargo.toml` files,
runs the pipeline on each project sequentially, logs failures, and continues to the next.
Skips `target/`, `node_modules/`, and `.git/` directories.

## Integrating Into Your Workflow

### Git hooks (local)

Drop fucina into your git hooks so every commit and push is clean.

**`.git/hooks/pre-commit`**
```sh
#!/bin/sh
set -eu
fucina
if ! git diff --quiet; then
    echo "Pipeline modified files. Re-add and retry."
    exit 1
fi
```

**`.git/hooks/pre-push`**
```sh
#!/bin/sh
set -eu
fucina
```

```bash
chmod +x .git/hooks/pre-commit .git/hooks/pre-push
```

### GitHub Actions

Add fucina as a CI step in any Rust project:

```yaml
- name: Install fucina
  run: cargo install fucina

- name: Run fucina
  run: fucina
```

Or pin to a release binary for faster CI (no compile):

```yaml
- name: Install fucina
  run: |
    curl -fsSL https://github.com/OWNER/fucina/releases/latest/download/fucina-linux-x64.tar.gz \
      | tar xz -C /usr/local/bin

- name: Run fucina
  run: fucina
```

### Makefile / justfile

```makefile
lint:
	fucina --no-test

check:
	fucina
```

### Pre-commit framework

If you use [pre-commit](https://pre-commit.com), add a local hook:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: fucina
        name: fucina
        entry: fucina
        language: system
        pass_filenames: false
```
