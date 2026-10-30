---
name: play-game
description: How to run the Battleships Python reference with scripted input and how to launch the Windows game, look at its window via a screenshot of that one window, and avoid typing into the user's other apps. Use to see a change working, reproduce a bug, or compare Python and Rust behaviour.
---

# Running and looking at the game (Windows)

Commands are Windows PowerShell 5.1 (see `local-checks` for its pitfalls). From a Git Bash tool wrap them in `powershell -NoProfile -Command` / `-File`. What was actually run is marked **verified** (checked 2026-10-30 on this machine); the rest is untested until the Rust game exists.

## 1. Python reference with scripted input (verified)

The game reads two lines per shot (row `1`-`10`, then column `A`-`J`). Feeding all 100 cells guarantees a finished game. Run from the repo root:

```powershell
$env:PYTHONUTF8 = '1'; $env:PYTHONDONTWRITEBYTECODE = '1'
[Console]::OutputEncoding = New-Object Text.UTF8Encoding($false)
$in = @(foreach ($r in 1..10) { foreach ($c in [char[]]'ABCDEFGHIJ') { "$r"; "$c" } }) + 'e'
$in | python -c "import random, battleshipsingle as b; random.seed(1); b.pelaa()" | Select-Object -Last 30
```

- `PYTHONUTF8=1` and the UTF-8 console encoding are needed: with piped output Python otherwise uses cp1252 and PowerShell decodes it as an OEM codepage, so `HÄVISIT` shows as `H─VISIT` (seen).
- `pelaa()` plays one game. `python battleshipsingle.py` with the same input also works; the trailing `e` answers the replay prompt.
- `random.seed(1)` gives the same game on Windows as on Linux (ends `HÄVISIT PELIN!`, 3171 output lines). Output contains the terminal clear sequence `\x1b[H...`; ignore it.
- The Python version is the reference only. It cannot be compared to Rust output line for line (different RNG), only by behaviour, see `game-spec`.

## 2. The Rust window game

Once built (`cargo build`, binary `target\debug\laivanupotus.exe`; window title `Laivanupotus`):

```powershell
$p = Start-Process target\debug\laivanupotus.exe -ArgumentList '--seed','1' -PassThru
Start-Sleep -Seconds 3                                  # let the window appear
powershell -NoProfile -ExecutionPolicy Bypass -File .claude\skills\play-game\window-shot.ps1 -ProcId $p.Id -Out "$env:TEMP\bs_shot.png"
Stop-Process -Id $p.Id
```

Then open the PNG with the Read tool to see it. Always stop the process afterwards.

### `window-shot.ps1`

[window-shot.ps1](window-shot.ps1) saves **one window** to a PNG and crops away the invisible resize border. Options: `-Title`, `-ProcId`, `-Out`, `-Screen`.

- Verified with a WinForms test window: by `-Title`, by `-ProcId` (identical output), a clean "Window not found" error, and Finnish `ÄÖäö` rendering. Run through `powershell.exe` with `-ExecutionPolicy Bypass`.
- **Not verified on an OpenGL/GPU game window.** If the PNG comes out black, retry with `-Screen` (copies the window's pixels from the desktop; the window must be visible and not covered, or other apps' pixels land in the picture, so do not use it casually).
- Capture only the game window. Never screenshot the whole desktop; it can show the user's private things.

### Prefer game-side dev flags over synthetic input

Build these into the game (`rust-port` step 3) so tests never need to fake the keyboard:

- `--seed <n>`: reproducible placement and AI.
- `--screenshot <path>`: draw a few frames, save a PNG with the framework's own frame capture, exit. More reliable than `window-shot.ps1` for GPU windows.
- Optionally `--moves "B3,C4,..."`: replay shots to reach a given state (mid-game, sunk ship, win, loss) without clicking.

Synthetic keystrokes or clicks (`SendKeys`, `SendInput`) go to whichever window has focus, which may be the user's editor or browser. Avoid them. If they are ever really needed, first confirm the foreground window is the game (`GetForegroundWindow` equals the game's handle) and tell the user.

## Cleanup

Delete screenshot PNGs and `__pycache__\`, stop game processes, and check `git status --short` shows only intended files.
