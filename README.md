Simple player vs computer battleships created with python. It's you versus computer. Who shall sail home to their families?

The game is being rebuilt in Rust as a small Windows game with a C64/Amiga look, playable with mouse and keyboard. Build it with `cargo build --release`; the result is the single file `target\release\klassikot8000.exe`. It opens a launcher where you pick the game, Battleships or 2048; `Esc` in a game returns to the launcher, and `Esc` in the launcher quits. The 2048 best score and unfinished game are saved in `%APPDATA%\klassikot8000\tallennus.txt`.

The launcher is in `src/kaynnistin/`, each game in its own folder under `src/pelit/` (`laivanupotus`, `luvut` = 2048), and code they share in `src/yhteinen/`.
