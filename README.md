# Audiobookshelf MPD TUI Client

A modern, intuitive Terminal User Interface (TUI) client for [Audiobookshelf](https://www.audiobookshelf.org/), built in Rust. It uses [MPD](https://www.musicpd.org/) for persistent, high-quality audio playback and [Ratatui](https://ratatui.rs/) for a rich terminal experience.

## Features

- **Intuitive TUI:** Clean and responsive interface with multiple views (Home, Library, Player).
- **Persistent Playback:** Leverages MPD to ensure your audio keeps playing even if the TUI is closed.
- **Audiobookshelf Integration:** Full support for libraries, personalized views (Continue Listening, Recently Added), and global search.
- **Vim-style Motions:** Navigate effortlessly with `hjkl` and other familiar keybindings.
- **Chapter Support:** View chapter lists and navigate directly to specific chapters.
- **Automatic Resume:** Picks up exactly where you left off, synced with your Audiobookshelf server.
- **Local Search Filtering:** Lightning-fast search results that update as you type.

## Prerequisites

- **Rust:** Latest stable version.
- **MPD:** A running Music Player Daemon instance (default: `localhost:6600`).
- **Audiobookshelf Server:** Access to an Audiobookshelf instance.

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/kemika180/abs-tui-client.git
   cd abs-tui-client
   ```

2. Build and run:
   ```bash
   cargo run
   ```

3. On the first run, you will be prompted to enter your Audiobookshelf server URL, username, and password. These credentials will be securely saved to your local config.

## Keybindings

### Global
- `q`: Quit the application (when not in an input field).
- `h`: Go to **Home** screen.
- `l`: Go to **Library** screen.
- `p`: Go to **Player** screen.

### Navigation (Home/Library/Chapters)
- `j` / `Down`: Move selection down.
- `k` / `Up`: Move selection up.
- `Enter`: Select item / Start playback.
- `Tab`: Switch libraries (Home) / Toggle chapter list (Player).

### Search (Library Screen)
- `i` / `Enter`: Enter **Insert Mode** to type.
- `Esc`: Return to **Normal Mode** (navigation).

### Playback (Player Screen)
- `Space` / `p`: Toggle Play/Pause.
- `,`: Step backward (default 30s).
- `.`: Step forward (default 30s).
- `<`: Jump to previous chapter.
- `>`: Jump to next chapter.

## Configuration

Configuration is stored in `~/.config/abs-tui-client/config.toml`. You can customize:

- `vim_motions`: Toggle Vim-style navigation (default: `true`).
- `step_seconds`: Change the seek interval for `,` and `.` (default: `30`).
- `mpd.address`: Customize your MPD connection string.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
