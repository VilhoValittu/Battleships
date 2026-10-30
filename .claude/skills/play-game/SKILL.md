---
name: play-game
description: How to launch and drive the Battleships game from the Bash tool, which has no TTY, using piped stdin for deterministic runs and tmux for interactive/TUI screens. Use to see a change working, reproduce a bug, or compare the Python and Rust versions.
---

# Running and driving the game

The Bash tool has no terminal, so `python3 battleshipsingle.py` or a TUI binary cannot be used directly. Two verified approaches (tmux 3.x and `script` exist; `expect` does not).

## 1. Scripted run through stdin (Python, deterministic)

The game reads two lines per shot: row (`1`-`10`), then column (`A`-`J`). Feeding all 100 cells guarantees the game ends, either by the player's win or the opponent winning first.

```bash
S=<scratchpad dir>
python3 - > $S/in.txt <<'EOF'
for r in range(1, 11):
    for c in "ABCDEFGHIJ":
        print(r); print(c)
print("e")   # answers the replay prompt if the game is run normally
EOF
PYTHONDONTWRITEBYTECODE=1 python3 -c "import random, battleshipsingle as b; random.seed(1); b.pelaa()" < $S/in.txt | tail -30
```

`pelaa()` plays one game without the replay prompt. Seeding makes the run repeatable (seed 1 ran to completion with an opponent win). `PYTHONDONTWRITEBYTECODE=1 python3 battleshipsingle.py < $S/in.txt` also works (verified: ends with `Lopetetaan peli.`), the trailing `e` answers the replay prompt. Use scratchpad files for the input, never the repo. If input runs out before a prompt is answered Python raises `EOFError`.

For Rust, make the logic seedable (see `rust-port`) and give the CLI milestone a way to take stdin the same way, so the same recipe works.

## 2. Interactive screens through tmux (verified with the Python game)

```bash
tmux kill-session -t bs 2>/dev/null
tmux new-session -d -s bs -x 100 -y 40 'python3 battleshipsingle.py'   # or: 'cargo run --quiet'
sleep 1
tmux send-keys -t bs 1 Enter A Enter          # keys one by one; Enter, Escape, Up, C-c, ...
sleep 0.5
tmux capture-pane -t bs -p | grep -v '^$'     # current screen as text
tmux kill-session -t bs
```

- The pane is taller than the content, so plain `tail` shows blank lines; filter with `grep -v '^$'` or capture everything.
- Add `-e` to `capture-pane` to keep colour escape codes when checking styling.
- Always `sleep` before capturing; the app needs a moment to redraw.
- Always kill the session afterwards. For a TUI that ignores `q`, use `tmux kill-session`.
- Try more than one pane size (for example `-x 80 -y 24` and `-x 120 -y 40`) for layout changes.
- The Rust recipe is the same but has not been run yet, because the Rust binary does not exist. Confirm it when it does and fix this file if it differs.

## Cleanup

`PYTHONDONTWRITEBYTECODE=1` avoids `__pycache__/`; if it appears anyway, delete it. Check `git status --short` shows only intended files.
