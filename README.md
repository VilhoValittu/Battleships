Simple player vs computer battleships created with python. It's you versus computer. Who shall sail home to their families?

The game is being rebuilt in Rust as a small Windows game with a C64/Amiga look, playable with mouse and keyboard. Build it with `cargo build --release`; the result is the single file `target\release\klassikot8000.exe`. It opens a resizable window sized to fit the screen, with a launcher where you pick the game: Battleships, 2048 or Solitaire (Klondike with three difficulty levels). `Esc` in a game returns to the launcher, and `Esc` in the launcher quits, as does the close button in the title bar at any time. The picture keeps its 16:9 shape at any window size. The 2048 best score and unfinished game are saved in `%APPDATA%\klassikot8000\tallennus.txt`.

The launcher is in `src/kaynnistin/`, each game in its own folder under `src/pelit/` (`laivanupotus`, `luvut` = 2048, `pasianssi`), and code they share in `src/yhteinen/`.
