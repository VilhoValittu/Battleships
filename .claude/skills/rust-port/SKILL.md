---
name: rust-port
description: Conventions and step order for rebuilding the Python Battleships game in Rust as a small, nice-looking native Windows game (environment facts, toolchain pin, layout, naming glossary, testing, scope limits). Use when creating or editing any Rust code, Cargo.toml or toolchain files, or deciding how to structure or extend the game.
---

# Rust rewrite: a nice little Windows game

The user's plan: `battleshipsingle.py` is a very simple terminal game; Claude rebuilds it in Rust as a **polished little game for Windows**. Development moved from WSL to native Windows. Read `game-spec` first, it defines the behaviour to match.

## Environment (checked 2026-10-30 from WSL through `powershell.exe`)

- Windows 11 (10.0.26100), Windows PowerShell **5.1** (no `pwsh`), Windows Terminal, VS Code, git 2.52, `gh`, `winget`. Intel UHD Graphics only: keep rendering to ordinary 2D, nothing GPU-hungry.
- Rust via rustup, three toolchains: `stable-x86_64-pc-windows-gnu` (1.95, the global default), `stable-x86_64-pc-windows-msvc` (1.95) and `1.96.0-x86_64-pc-windows-msvc`. A hello-world built and ran on all three. MSVC link works: VS 2022 BuildTools and Windows SDK 10.0.26100.0 are installed. clippy and rustfmt exist in the gnu and the stable msvc toolchain.
- Python 3.13 (`python`, `py`) with `pytest` only; the game is Python-reference only.
- Remotes: `origin` = VilhoValittu/Battleships (push here), `upstream` = anatt1b/Battleships (do not push).
- The Linux/WSL copy at `/home/ville/dev/Battleships` is an old working copy. On Windows work in a normal drive path, e.g. `C:\dev\Battleships`, **not** through `\\wsl.localhost\...` (cmd.exe cannot use UNC working directories and builds there are slow).

## Toolchain pin

Pin the project so it never depends on the global default (currently gnu): `rust-toolchain.toml` at the repo root:

```toml
[toolchain]
channel = "stable-x86_64-pc-windows-msvc"
components = ["clippy", "rustfmt"]
```

Verified: with this file `rustc -vV` reports host `x86_64-pc-windows-msvc` and clippy/rustfmt run. MSVC is the best-supported target for Windows game and window crates. Edition 2024.

## Scope limits

Windows x86_64 only. No Linux/macOS/wasm/mobile targets, no CI workflows, Docker, installers or telemetry unless asked. `cargo build --release` producing an `.exe` is fine; how the game gets distributed is undecided. Leave `battleshipsingle.py` untouched as the reference until the user says to remove it. Do not push, and do not commit unless asked.

## "Fancy" means a real window

The terminal-UI idea is dropped. Default proposal: a windowed 2D game with **`macroquad`** (small, quick to build, simple drawing/input/audio). Alternative if the user wants an engine: `bevy` (much heavier compile). Adding the framework is a big decision: state the choice and why, and get the user's okay before the first dependency, unless they already picked one. Keep the logic library free of framework types so the choice stays swappable.

Details the window version has to get right:

- Fixed window title `Laivanupotus`, so `play-game/window-shot.ps1` can find it.
- Finnish text must render: check `ä ö Ä Ö` (for example `HÄVISIT PELIN!`, `Hävittäjä`) in a screenshot; bundle a TTF in `assets/` if the default font lacks them.
- Dev flags on the game binary instead of synthetic keystrokes: `--seed <n>` (reproducible placement and AI) and `--screenshot <path>` (draw a few frames, save a PNG, exit). Details in `play-game`.

## Step order

1. `cargo init` in the repo root (package `laivanupotus`), add `rust-toolchain.toml` (above), `.gitignore` (`/target`, `__pycache__/`), `.gitattributes` with `*.rs text eol=lf`, `*.toml text eol=lf`, `Cargo.lock text eol=lf` (the Python file and `.vscode/pyproject.toml` are stored CRLF; do not renormalize them).
2. **Logic library with tests, no I/O and no drawing.** Port `sijoita_laivat`, `kasittele_laukaus`, `kaikki_upotettu`, `ruudun_nimi`, the AI shot choice. Randomness is injected (`impl rand::Rng`), never a global RNG inside logic, so tests seed `StdRng::seed_from_u64`.
3. **Window with the same flow:** random placement, click to shoot, same Finnish result messages, win/loss text, play-again. Click handling replaces the row/column prompts.
4. **Polish and upgrades**, one named step at a time, each with tests and a screenshot: animations, sound, manual ship placement, hunt-mode AI, sunk-ship reveal. Update `game-spec` when behaviour changes on purpose.

## Layout

```
Cargo.toml  rust-toolchain.toml  .gitignore  .gitattributes
assets/            // font, sounds, images
src/lib.rs         // re-exports; pure game logic
src/ruutu.rs       // Ruutu { rivi, sarake }, parsing "B10", Display
src/laivasto.rs    // Laiva, Laivasto, placement, Laukaustulos, kasittele_laukaus
src/vastustaja.rs  // AI shot choice
src/main.rs        // window, loop, CLI flags
src/ui/            // drawing, mouse/keys, message texts (Finnish text lives here)
tests/             // integration tests over the public lib API
```

## Naming

Mirror the Python's Finnish domain words so old and new code stay greppable side by side. This default comes from the existing code; if the user prefers English identifiers, switch once and update this file.

| Python | Rust |
|---|---|
| `KOKO`, `SARAKKEET`, `LAIVAT` | `const KOKO: usize`, `SARAKKEET: &str`, `LAIVAT: [(&str, usize); 5]` |
| `(rivi, sarake)` tuple | `struct Ruutu { rivi: u8, sarake: u8 }` (0-based, `Copy, Eq, Hash`) |
| `sijoita_laivat()` | `Laivasto::satunnainen(&mut impl Rng)` |
| `kasittele_laukaus(...)` | `Laivasto::kasittele_laukaus(&mut self, Ruutu) -> Laukaustulos` (`Ohi`, `Osuma`, `Upposi(&str)`) |
| `kaikki_upotettu(...)` | `Laivasto::kaikki_upotettu()` |
| `vastustajan_laukaus(...)` | `vastustaja::valitse(&Ammutut, &mut impl Rng) -> Ruutu` |
| `pelilauta`, `laukaus`, `tulos_teksti` | UI layer, not logic |

Snake_case functions, CamelCase types, ASCII-only identifiers (no ä/ö, as in the Python). Game text stays Finnish, copied from `game-spec`. Code comments follow the repo: Finnish, short, why not what. Talk to the user in Finnish.

## Testing

Python's RNG cannot be matched, so test **invariants** over many seeds: five ships with the lengths in `LAIVAT`, straight, in bounds, no overlap; a ship is `Upposi` exactly on its last cell; repeat shots are rejected; the AI never picks a shot cell and eventually covers all 100; win is detected on the 17th distinct ship hit. Drawing is checked by screenshot, not unit tests. Run everything through `local-checks`.

## Style

Plain structs/enums and small functions, like the Python. `Result`/`Option` over panics for input; panics only for broken invariants. No `unsafe`, no `unwrap` on user input. Add a dependency only when it earns its place, and say which and why.
