# Klassikot8000

[Suomeksi](README.md) | **English**

Bored? Six old-school games in one small window, dressed up in C64 and Amiga style. No account, no ads, nothing to learn. Open it, pick a game and let the time pass.

Runs even on a potato.

## Games

### Battleships
You against the computer. Each side has five ships, and you take turns firing until one fleet lies on the bottom. Who sails home to their family?

*Mouse, or arrows and space.*

### 2048
Push the numbers around the board, merge equal ones and chase 2048. One more move and then you'll stop. Maybe. An unfinished game waits for next time.

*Arrows or mouse.*

### Solitaire
The familiar Klondike that has eaten thousands of working days. Three levels: on Easy you draw one card at a time and can undo, on Hard you draw three, get three passes through the stock and can't undo anything.

*Mouse, or arrows and space.*

### Laatan murskain
Keep the ball in play and break the bricks. Five stages, three balls, and a fast clear earns bonus points. A white brick brings a second ball and grows the paddle, and as long as both balls stay in play, bricks score double. Watching two balls at once is exactly as hard as it sounds.

*Mouse or arrows, launch with space or a click.*

### Asteroids
A lonely ship, endless rocks. Big rocks split into smaller ones, flying saucers shoot back, and when things get impossible you can always jump blindly somewhere else. You have three ships, and every 10,000 points gives you a new one.

*Arrows or mouse, space fires, H jumps.*

### Mato8000
A worm game where hitting yourself is the whole idea. A hit closes a loop, and everything caught inside it is yours. The loop uses up the worm's length, though, so eat some apples now and then. The more you catch at once, the bigger the multiplier. Watch out for walls and mosquitoes.

*Arrows, WASD or mouse.*

## Good to know

- `Esc` returns from a game to the menu, `P` pauses the action games, `M` mutes the sound and `L` in the menu switches the language between Finnish and English.
- All games except Battleships and Solitaire also have an online Top 3. Without a connection the games play just as well.

## Get playing

Grab the package for your machine from the [releases](https://github.com/VilhoValittu/Klassikot8000/releases/latest) and unpack it. The download is under 1 MB for Windows and under 3 MB for Linux.

- **Windows** (`windows-x64`): run `klassikot8000.exe`.
- **Linux** (`linux-x64`, `linux-arm64`): run `klassikot8000`. The game needs X11 or XWayland, plus the ALSA library for sound; desktop machines usually have these already.
- **macOS** (`macos-universal`, Intel and Apple Silicon): run `Klassikot8000.app`. The package isn't signed by Apple, so macOS blocks the first launch. Remove the block in Terminal: `xattr -d com.apple.quarantine Klassikot8000.app`.
