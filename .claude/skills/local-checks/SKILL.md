---
name: local-checks
description: The exact local commands to format-check, lint, test and smoke-run the Battleships project on Windows (Rust game and Python reference), in order, with what is and is not installed and the PowerShell 5.1 pitfalls. Use after changing any .rs or .py file, before reporting work as done or committing.
---

# Local verification (Windows)

Run from the repo root. There is no CI. Report honestly which steps ran and which were skipped.

Shell: Windows PowerShell **5.1** is the only PowerShell installed (no `pwsh`). In 5.1:

- there is no `&&`; run steps on separate lines and check `$LASTEXITCODE`;
- do **not** add `2>&1` to cargo: cargo writes progress to stderr and 5.1 then prints it as red `NativeCommandError` noise even on success (seen). The exit code is what counts;
- `'A'..'J'` char ranges do not work; use `[char[]]'ABCDEFGHIJ'`;
- if the Bash tool is Git Bash instead, `&&` works and cargo lines are identical, but run `.ps1` helpers via `powershell -NoProfile -ExecutionPolicy Bypass -File <script>`.

## Rust (once `Cargo.toml` exists)

`rust-toolchain.toml` pins `stable-x86_64-pc-windows-msvc` (see `rust-port`). MSVC linking works from a plain PowerShell (VS 2022 BuildTools + Windows SDK are installed). Order, stop at the first failure:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Fix formatting with `cargo fmt --all`. Tests never exercise the window: for any UI change also run the game and look at it (`play-game`). `cargo-nextest` is not assumed; use `cargo test`.

## Python reference (`battleshipsingle.py`)

Windows Python 3.13 (`python`) has `pytest` 9 but **not** `ruff`, `black` or `mypy`; there is no `.venv` and no `tests/` directory. Do not install tools without asking the user. Config lives at `.vscode/pyproject.toml` (not the repo root): line length 88, black, ruff rules `E F I UP B SIM PERF`, `E501` ignored. If they ever get installed, pass it explicitly:

```powershell
python -m ruff check --config .vscode/pyproject.toml battleshipsingle.py
python -m black --config .vscode/pyproject.toml --check battleshipsingle.py
```

Works today:

```powershell
python -c "import ast; ast.parse(open('battleshipsingle.py', encoding='utf-8').read())"   # syntax check
```

then the scripted run from `play-game`. `py_compile` is avoided: it writes `__pycache__/` regardless of `PYTHONDONTWRITEBYTECODE`.

`battleshipsingle.py` and `.vscode/pyproject.toml` are stored with CRLF line endings. Keep them when editing; do not convert.

## After running

- `git status --short` shows only intended changes. Delete stray `__pycache__\` and screenshot PNGs; stop leftover game processes (`Stop-Process -Name laivanupotus -ErrorAction SilentlyContinue`).
- `target\` must stay out of git (`.gitignore`).
- Do not commit or push unless the user asks. Push only to `origin`.
