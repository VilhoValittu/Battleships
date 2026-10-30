---
name: rust-port
description: Conventions and step order for rewriting the Python Battleships game in Rust and making it fancier (layout, naming glossary, crate choices, tests, scope limits). Use when creating or editing any Rust code, Cargo.toml, or deciding how to structure or extend the game.
---

# Rust rewrite of Battleships

Context from the user: the repo is a very simple Python game; Claude will rebuild it in Rust and make it fancier. Development happens **locally on this machine only** (WSL2 Linux, Rust stable 1.96, `cargo`/`rustfmt`/`clippy` installed). Read `game-spec` first, it defines the behaviour to match.

## Scope limits (local-only)

No CI workflows, Docker, release/installer packaging, wasm, mobile or cross-compilation targets, and no telemetry/network. Do not add them unless asked. Do not delete or edit `battleshipsingle.py` or touch `.vscode/` (untracked, the user's editor setup); the Python file stays as the reference until the user says to remove it.

"Fancier" is not defined yet. Default assumption: a terminal UI, since the game is already a terminal game and the machine is WSL2 (a GUI would need WSLg). Before building UI beyond a plain port, ask the user what they mean (colours/animation in the terminal, mouse, sound, a real window) unless they already said.

## Step order

1. `cargo init` in the repo root (edition 2024, package `laivanupotus`), plus a `.gitignore` with `/target` and `__pycache__/` (none exists today). Do not commit unless asked.
2. **Logic library with tests, no I/O.** Port `sijoita_laivat`, `kasittele_laukaus`, `kaikki_upotettu`, `ruudun_nimi`, the AI shot choice. Randomness is injected (`impl rand::Rng`), never `thread_rng()` inside logic, so tests can seed `StdRng::seed_from_u64`.
3. **Plain 1:1 CLI** with the same prompts and Finnish strings from `game-spec`. Compare it against the Python version by playing both (see `play-game`).
4. **Fancy UI** on top (default: `ratatui` + `crossterm`). Logic never imports UI types.
5. **Behaviour upgrades** (hunt-mode AI, manual placement, ...), one at a time, each a separate named step with tests. Update `game-spec` when behaviour changes deliberately.

## Layout

```
Cargo.toml
src/lib.rs        // re-exports; pure game logic, no printing, no stdin
src/ruutu.rs      // Ruutu { rivi, sarake }, parsing "B10", Display
src/laivasto.rs   // Laiva, Laivasto, placement, Laukaustulos, kasittele_laukaus
src/vastustaja.rs // AI shot choice
src/main.rs       // wires logic + UI
src/ui/           // input prompts, board rendering, messages (Finnish text lives here)
tests/            // integration tests over the public lib API
```

## Naming

Mirror the Python's Finnish domain words so the two codebases stay greppable side by side. This is a default taken from the existing code; if the user prefers English identifiers, switch once and update this file.

| Python | Rust |
|---|---|
| `KOKO`, `SARAKKEET`, `LAIVAT` | `const KOKO: usize`, `SARAKKEET: &str`, `LAIVAT: [(&str, usize); 5]` |
| `(rivi, sarake)` tuple | `struct Ruutu { rivi: u8, sarake: u8 }` (0-based, derive `Copy, Eq, Hash`) |
| `sijoita_laivat()` | `Laivasto::satunnainen(&mut impl Rng)` |
| `kasittele_laukaus(...)` | `Laivasto::kasittele_laukaus(&mut self, Ruutu) -> Laukaustulos` (`Ohi`, `Osuma`, `Upposi(&str)`) |
| `kaikki_upotettu(...)` | `Laivasto::kaikki_upotettu()` |
| `vastustajan_laukaus(...)` | `vastustaja::valitse(&Ammutut, &mut impl Rng) -> Ruutu` |
| `pelilauta`, `laukaus`, `tulos_teksti` | UI layer, not logic |

Snake_case functions, CamelCase types, ASCII only (no ä/ö in identifiers, as in the Python). Game text stays Finnish and is quoted from `game-spec`, not retranslated. Comments follow the repo: Finnish, short, explaining why. Talk to the user in Finnish.

## Testing

Python's RNG cannot be matched, so test **invariants**, not sequences, over many seeds: five ships with the lengths in `LAIVAT`, all straight, in bounds, no overlap; a ship is `Upposi` exactly on its last cell; repeat shots are rejected; the AI never picks an already shot cell and covers all 100 cells eventually; win detected on the 17th distinct ship hit. For the UI use `ratatui::backend::TestBackend` buffers, with hand-built boards (rendering takes ships + shots as input, so it needs no RNG). Run everything through `local-checks`.

## Style

Prefer plain structs/enums and small functions, as the Python does. `Result`/`Option` over panics for user input; panics only for broken invariants. No `unsafe`, no `unwrap` on user input. Add a dependency only when it earns its place, and say which one and why when adding it.
