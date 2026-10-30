---
name: local-checks
description: The exact local commands to lint, format-check, test and smoke-run the Battleships project (Python reference and Rust rewrite), in order, with what is and is not installed. Use after changing any .py or .rs file, before reporting work as done or committing.
---

# Local verification

Everything runs on this machine from the repo root `/home/ville/dev/Battleships`. There is no CI. Report honestly which steps ran and which were skipped.

## Rust (once `Cargo.toml` exists)

Installed and working: `cargo`, `rustfmt`, `clippy` (stable 1.96). Run in this order and stop at the first failure:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Fix formatting with `cargo fmt --all`. Then smoke-test the real binary, since unit tests never exercise the terminal: see `play-game`. For UI changes, a green `cargo test` is not enough; look at the screen.

`cargo-nextest` is not installed; use plain `cargo test`.

## Python (`battleshipsingle.py`, reference only)

Configured but **not installed** in the system `python3` 3.12: `ruff`, `black`, `pytest`, `mypy`. There is no `.venv` (the editor config points at one) and no `tests/` directory. Do not install tools without asking the user.

Config lives at `.vscode/pyproject.toml` (not the repo root), so tools need it passed explicitly if they are ever installed: line length 88, black formatting, ruff rules `E F I UP B SIM PERF` with `E501` ignored.

```bash
ruff check --config .vscode/pyproject.toml battleshipsingle.py       # if installed
black --config .vscode/pyproject.toml --check battleshipsingle.py    # if installed
```

What works today without installing anything:

```bash
python3 -c "import ast; ast.parse(open('battleshipsingle.py').read())"   # syntax check
```

(`py_compile` is avoided: it writes `__pycache__/` even with `PYTHONDONTWRITEBYTECODE=1`.)

then the scripted run from `play-game`. If you edited the Python, say that ruff/black were not available.

## After running

- `git status --short` should show only intended changes. Remove stray `__pycache__/` and kill leftover tmux sessions (`tmux kill-session -t bs`).
- `target/` must stay out of git (`.gitignore`).
- Do not commit or push unless the user asks.
