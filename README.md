<table width="100%">
<tr>
<td align="center" valign="top" width="15%">

<img src="assets/images/Low_Noise_Logo.png"
     alt="Low Noise"
     width="90">

</td>

<td align="center" valign="top">

<img src="assets/images/RetroTUI_Logo.png"
     alt="RetroTUI Games"
     width="400">

</td>
</tr>
</table>

<h1 align="center">
RetroTUI Games
</h1>

<p align="center">
<b>A retro gaming arcade machine inside your terminal.</b>
</p>

<p align="center">
Minimal. Fast. Native. Terminal.<br>
Built by <b>Low Noise</b>.
</p>

# RETRO HUB

> A retro gaming arcade machine right inside your terminal.

**Retro Hub** is a TUI (Terminal User Interface) application that transforms your terminal into an old-school arcade cabinet. It features a curated collection of classic games, all running natively in the terminal with zero dependencies beyond what your terminal already provides.

---

## Features

- **Arcade Cabinet Aesthetic** — a decorative double-line frame with "RETRO HUB" title, scan-line decorations, and an animated starfield that orbits gently behind the cabinet
- **8 Classic Games**: Pong, Snake, Chess, Tetris, Breakout, Minesweeper, Sudoku, and 2048
- **4 Color Themes**: Tokyo Night, Catppuccin, Gruvbox, and Nord
- **Dynamic Resizing** — every game and the menu adapt seamlessly when you resize the terminal
- **Settings panel** to switch themes on the fly
- **Smooth 60 FPS** game loop with frame pacing
- **Full keyboard control** — no mouse needed

---

## How it see?
<img src="assets/images/MainScreen.png" width="220">

---

## Games

<table>
<tr>
<td align="center" width="25%">

<img src="assets/images/PongGameScreen.png" width="220">

### Pong

Classic table-tennis. Compete against an AI opponent.

</td>

<td align="center" width="25%">

<img src="assets/images/SnakeGameScreen.png" width="220">

### Snake

Eat apples, grow longer and survive.

</td>

<td align="center" width="25%">

<img src="assets/images/TetrisGameScreen.png" width="220">

### Tetris

Stack tetrominoes and clear lines.

</td>
</tr>

<tr>
<td align="center">

<img src="assets/images/BreakoutGameScreen.png" width="220">

### Breakout

Destroy all the bricks.

</td>

<td align="center">

<img src="assets/images/MinesweeperGameScreen.png" width="220">

### Minesweeper

Five difficulty levels.

</td>

<td align="center">

<img src="assets/images/SudokuGameScreen.png" width="220">

### Sudoku

Generate and solve puzzles.

</td>

<td align="center">

<img src="assets/images/2048GameScreen.png" width="220">

### 2048

Reach the 2048 tile.

</td>
</tr>
</table>

---

## Installation

### Prerequisites

- Rust toolchain (edition 2021)
- A terminal with UTF-8 and true-color support (almost all modern terminals)

### From source

```bash
git clone https://github.com/yourusername/retrohub.git
cd retrohub
cargo run --release
```

### Install globally

```bash
cargo install --path .
retrohub
```

---

## Controls

### Global

| Key | Action |
|---|---|
| `Esc` | Back / Exit game / Quit app |
| `↑ ↓` | Navigate menu |
| `Enter` | Select game / Confirm |
| `Tab` | Toggle submenu |

### Pong

| Key | Action |
|---|---|
| `↑ ↓` / `W S` | Move paddle |
| `Space` | Pause / Restart |

### Snake

| Key | Action |
|---|---|
| `↑ ↓ ← →` / `W A S D` | Change direction |
| `Space` | Pause / Restart |

### Chess

| Key | Action |
|---|---|
| `↑ ↓ ← →` | Move cursor |
| `Enter` | Select piece / Make move |
| `Esc` | Exit |

### Tetris

| Key | Action |
|---|---|
| `← →` | Move piece |
| `↑` | Rotate |
| `↓` | Soft drop |
| `Space` | Hard drop |
| `C` | Hold piece |
| `P` | Pause |

### Breakout

| Key | Action |
|---|---|
| `← →` / `A D` | Move paddle |
| `Space` | Launch ball / Pause / Restart |

### Minesweeper

| Key | Action |
|---|---|
| `↑ ↓ ← →` | Move cursor |
| `Enter` / `Space` | Dig / Flag |
| `F` | Toggle dig/flag mode |
| `1`–`5` | Select difficulty |

### Sudoku

| Key | Action |
|---|---|
| `↑ ↓ ← →` | Move cursor |
| `1`–`9` | Place number |
| `Backspace` | Clear cell |
| `P` | Toggle pencil mode |
| `H` | Hint |
| `N` | New puzzle |

### 2048

| Key | Action |
|---|---|
| `↑ ↓ ← →` | Slide tiles |
| `R` | Restart |
| `U` | Undo |
| `Space` | Continue after win |

---

## Themes

Retro Hub ships with four carefully crafted color schemes:

- **Tokyo Night** — deep blue-grey background with electric blue accents
- **Catppuccin** — warm mauve-grey with pastel accents
- **Gruvbox** — earthy retro tones inspired by vim's gruvbox
- **Nord** — arctic blue-steel palette

Switch themes from the in-app Settings menu or by pressing `Tab` → `Settings`.

---

## Architecture

```
src/
├── main.rs              # Entry point, game loop, arcade cabinet rendering
├── engine/
│   ├── mod.rs           # Engine struct (theme, events, audio, etc.)
│   ├── renderer.rs      # Low-level drawing functions, starfield, arcade cabinet
│   ├── scene.rs         # Scene trait and SceneManager
│   ├── input.rs         # Keyboard input polling
│   ├── theme.rs         # Color themes (ColorScheme, ThemeManager)
│   ├── events.rs        # Event bus
│   ├── animation.rs     # Tweening engine
│   ├── audio.rs         # Audio (beep via terminal bell)
│   ├── layout.rs        # Layout helpers (centered_rect, stacks)
│   ├── widgets.rs       # Reusable UI widgets
│   ├── timing.rs        # Timer and GameLoop
│   └── resources.rs     # ASCII art storage
├── games/               # Game implementations
│   ├── mod.rs           # Game registry
│   ├── pong.rs
│   ├── snake.rs
│   ├── chess.rs
│   ├── tetris.rs
│   ├── breakout.rs
│   ├── minesweeper.rs
│   ├── sudoku.rs
│   └── game2048.rs
├── ui/
│   ├── mod.rs           # MainMenuScene, SettingsScene
│   └── menu.rs          # SplashScene with animated logo
├── settings.rs          # Settings manager
├── profile.rs           # Player profile
├── achievements.rs      # Achievement system
├── leaderboard.rs       # Score tracking
└── save.rs              # Save/load state
```

### Key design decisions

- **No immediate-mode widgets** — every pixel is drawn manually to the buffer for full control over the retro aesthetic
- **Scene trait** — each screen (menu, game, settings) implements `Scene` with `render`, `update`, and `handle_key` methods
- **Arcade cabinet as overlay** — the frame, starfield, and decorative elements are drawn in `main.rs` around the scene content, not inside each scene
- **All games are resizable** — they query the available area each frame and clamp their internal coordinates

---

## Technologies

- **[Ratatui](https://github.com/ratatui-org/ratatui)** — terminal UI framework
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** — cross-platform terminal manipulation
- **[Rand](https://github.com/rust-random/rand)** — random number generation
- **[Serde](https://serde.rs/)** — serialization for settings and save data

---

## Roadmap

- Additional color themes
- Persistent high scores between sessions
- Sound effects via terminal bell
- More games (Space Invaders, Pac-Man-style maze runner)
- Leaderboard per game
- Achievement badges

---

## License

MIT

---

## Contributing

Contributions are welcome. Open an issue or pull request on GitHub.

---

*Made with Rust and retro spirit.*

