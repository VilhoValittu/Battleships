Simple player vs computer battleships created with python. It's you versus computer. Who shall sail home to their families?

The game is being rebuilt in Rust as a small Windows game with a C64/Amiga look, playable with mouse and keyboard. Build it with `cargo build --release`; the result is `target\release\klassikot8000.exe`. It opens a resizable window sized to fit the screen, with a launcher where you pick the game: Battleships, 2048, Solitaire (Klondike with three difficulty levels), Laatan murskain (Breakout, five levels), Asteroids (endless waves) or Mato8000 (a snake game where hitting yourself is the point: the loop you close captures everything inside it, at the price of the length it used). `Esc` in a game returns to the launcher, and `Esc` in the launcher quits, as does the close button in the title bar at any time. The picture keeps its 16:9 shape at any window size. The 2048 best score and unfinished game, the Laatan murskain, Asteroids and Mato8000 best scores and the language are saved in `%APPDATA%\klassikot8000\tallennus.txt`.

The game is in Finnish and English: press `L` or the button in the launcher to switch. All texts are in `kielet.txt`, read at startup from next to the exe, so they can be edited without rebuilding. The exe has a built-in copy, so it also runs on its own and any text missing from the file comes from that copy.

The launcher is in `src/kaynnistin/`, each game in its own folder under `src/pelit/` (`laivanupotus`, `luvut` = 2048, `pasianssi`, `tiilirikko` = Laatan murskain, `asteroidit` = Asteroids, `silmukka` = Mato8000), and code they share in `src/yhteinen/`.

2048, Laatan murskain, Asteroids and Mato8000 have online top 3 lists served by the Cloudflare Worker in `palvelin/` (Worker + D1). The game falls back to the local list when offline and resends unsent scores on the next launch from the save file. Scores are not protected against cheating beyond input checks, a maximum score and a rate limit. Run the commands from `palvelin/`:


