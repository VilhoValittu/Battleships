---
name: game-spec
description: Reference spec of the Battleships game as implemented in battleshipsingle.py (rules, fleet, turn order, input validation, AI, every user-visible Finnish string). Use when porting, changing or testing game logic or text, or when you need to know what the original game does before touching the Rust rewrite.
---

# Battleships reference spec

`battleshipsingle.py` (174 lines, stdlib only) is the **source of truth** for behaviour until the user says otherwise. Re-read the file if this spec and the code disagree; the code wins, then fix this file.

Code, comments and all UI text are Finnish. Identifiers are ASCII-fied Finnish (`kasittele`, `nayta`), no ä/ö. The file is stored with CRLF line endings; keep them if you edit it. The Rust target is a native Windows window game (see `rust-port`); the last section says what changes with that.

## Rules

- Board 10x10. Columns `A`-`J` (`SARAKKEET`), rows `1`-`10`. Internally `(rivi, sarake)`, both 0-based.
- Fleet, both sides, fixed order in `LAIVAT`: Lentotukialus 5, Taistelulaiva 4, Hävittäjä 3, Sukellusvene 3, Partioalus 2 (17 cells).
- Placement is random (`sijoita_laivat`): pick horizontal `(0,1)` or vertical `(1,0)`, pick a start that keeps the ship on the board, retry until it does not overlap a placed ship. The Python lets ships touch; **the Rust game forbids it by the user's request (2026-10-30)**: ships keep at least one empty cell between them, also diagonally. The player does not place their own ships.
- Two boards. `pelaajan_ammutut` = shots the player fired at the opponent's board; `vastustajan_ammutut` = shots the opponent fired at the player's board.
- Turn order: player shoots, win check, opponent shoots, loss check. The player always goes first, and if the player sinks the last ship the opponent does **not** get a reply shot.
- A ship sinks when all its cells are in the shot set. `kasittele_laukaus` reports the ship name only on the shot that completes it; earlier hits on it report plain hit.
- No hit-again-bonus turns: a hit does not give an extra shot.

## Player input (`laukaus`)

- Asks row first (`Anna rivi numero 1-10: `), then column (`Anna sarake kirjain A-J: `).
- Row must be exactly one of `"1"`..`"10"` after `.strip()` (so `01` and empty are invalid). Error: `Syötä validi numero!`, then re-ask the row.
- Column: `.strip().upper()`, exactly one letter in `A-J`. Error: `Syötä validi kirjain!`, then re-ask the column.
- Already-shot cell: `Ammuit jo tähän!` and **both** prompts restart.

## Opponent AI (`vastustajan_laukaus`)

Uniform random choice among cells not yet shot. No memory of hits, no hunt/target mode.

## Output

Per round: `tulosta_ruudut` prints the banner, the opponent board (`VASTUSTAJAN LAUTA (ammuntasi):`, ships hidden), then the own board (`OMA LAUTA (# = laiva, X = osuma, - = ohi):`, ships shown), then the previous round's message.

Cell glyphs (`pelilauta`): `X` shot and ship, `-` shot and water, `#` unshot ship (own board only), space otherwise. Rendering: row label `{n:2}|`, each cell ` {glyph} |`, header ` A   B ...`. Sunk ships are not revealed any further on the opponent board.

After the player enters a shot the screen and scrollback are cleared (`\033[H\033[2J\033[3J`, a terminal-only trick), then the result message is shown with the next board redraw.

Message strings (`tulos_teksti`, player first line begins with a blank line):

| Case | Text |
|---|---|
| miss | `{ampuja} {ruutu}: ohi.` |
| hit | `{ampuja} {ruutu}: OSUMA!` |
| sink | `{ampuja} {ruutu}: OSUMA! {laiva} upposi!` |

`ampuja` is `Sinä ammuit` or `Vastustaja ampui`; `ruutu` is `{kirjain}{rivi}` such as `B10`.

Endings (appended to the message, boards drawn once more): win `****HIHIHIHIII VOITIT PELIN!****`, loss `HÄVISIT PELIN!`. The opponent's unhit ships are **not** revealed at the end, even after a loss.

Start of every game: a separator line, `Tervetuloa pelaamaan laivanupotusta.`, rules lines, separator. Replay prompt: `Haluatko pelata uudestaan? k = kyllä, e = ei : `. Only an exact lowercase `k` continues (no strip, no `K`); anything else prints `Lopetetaan peli.` and exits.

## What changes in a window, and what must not

Interface mechanics change necessarily: typed row/column prompts and their errors (`Syötä validi numero!`, `Syötä validi kirjain!`) become mouse clicks; clicking an already shot cell does nothing (the `Ammuit jo tähän!` rule still holds, show it as feedback or ignore the click); the ASCII board becomes graphics with the same meanings (unknown, miss, hit, own ship); the clear-screen trick and the `k`/`e` replay prompt become a redraw and a button or key.

Everything else carries over unchanged: fleet and placement rules, turn order and the no-reply-after-win rule, sink reporting on the completing shot only, the random AI, and the Finnish message texts (`ohi.`, `OSUMA!`, `... upposi!`, win and loss lines).

Implemented window controls: intro screen before every game (Space, Enter or click starts); the cursor moves with mouse hover, arrows, `A`-`J` and `1`-`0` (`0` = row 10); click, Space or Enter shoots; a repeat shot shows `Ammuit jo tähän!`; at the end `K` or its button replays, `E` or its button quits; `Esc` quits at any time.

Added by the user's request (2026-10-30), not in the Python: a fleet statistic under each board, `Upotettu n/5` and all five ship names with their length. A sunk ship turns orange and struck through, with red length blocks. It shows only sunk or not sunk, never hits on a floating ship, so it gives away nothing about the opponent beyond the `upposi!` messages.

Added by the user's request (2026-10-30), not in the Python: Doom-spirited sound effects, synthesized in `src/ui/aanet.rs` (no Doom assets or sound files). Door when a game starts, shotgun for the player's shot, fireball for the opponent's, then splash (miss), explosion (hit) or big explosion with a demon death rattle (sink); a grunt for a repeat shot; own metal riff on a win, descending chords and a scream on a loss. On by default. `M` or the `Ääni:` switch at the top right toggles them in any phase, silencing playing and queued sounds at once; the choice survives a replay but not a restart. With no audio device the game runs silently.

Added by the user's request (2026-10-30), not in the Python: C64/Amiga-style vector graphics at 960x768. A rotating 3D battleship over a perspective sea on the intro screen, vector ships on the own board, and an animated shell flight and explosion or splash for each shot, timed with the sounds. A shot's cell mark appears on impact; the opponent's message, and the win or loss line with its buttons, wait for the opponent's shot to land. The buttons and `K`/`E` already work during that short wait. Enemy ships stay hidden as before; sunk-ship reveal is still not implemented.

## Gaps that are safe to improve (only when the user has asked for the feature)

- Intro text hard-codes "5 laivaa (pituudet 5, 4, 3, 3 ja 2)" instead of deriving from `LAIVAT`.
- Random AI with no hunt mode; manual ship placement; sunk-ship reveal; reveal enemy fleet at game end; `K`/whitespace accepted at replay.
- Do not "fix" any of these silently during a straight port. Port first, then change behaviour as a separate, named step.
