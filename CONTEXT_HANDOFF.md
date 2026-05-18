# Context Handoff: Audiobookshelf MPD TUI Client

## Objective
Build a new Audiobookshelf client in Rust using `ratatui` for an intuitive TUI and `mpd` for persistent audio playback.

## Project Details
- **Location:** `/home/jessica/Development/abs-tui-client`
- **Tech Stack:** Rust (Edition 2024), Ratatui, Tokio, Reqwest, Mpd_client, Directories, Config, Toml.
- **Key Decisions:**
  - **Local Filtering for Search:** Fetches all library items once and filters locally. This avoids version-dependent API issues and ensures instantaneous results as the user types.
  - **Chapter-Centric Progress:** The Player UI and Progress Bar focus on the current chapter's duration and percentage.
  - **Vim-Style Navigation:** hjkl for lists, `i`/`Enter` for Insert Mode (search), `,`/`.` for stepping, `<`/`>` for chapters.
  - **Stateful Scrolling:** All lists use `ListState` and `render_stateful_widget` for automatic scrolling.
  - **Robust Models:** Models use `Option` extensively to handle incomplete metadata from various Audiobookshelf server versions.

## Current Progress
1. **Playback Orchestration:**
   - `play_book` in `app.rs` handles starting sessions, clearing MPD queue, and adding streamable URLs with token auth.
   - Resumes playback from the server's `currentTime`.
2. **Real-time Status:**
   - Main loop polls MPD status every 100ms.
   - `App` tracks `playback_status` and uses it for live UI updates.
3. **Search & Library:**
   - Instant search results in "Insert Mode".
   - Library cycling via `Tab` on the Home screen.
   - Smooth selection logic that skips headers and handles optional book IDs.
4. **Player UI:**
   - Displays Book Title, Author, Chapter Title, and Chapter Number.
   - Chapter-specific Progress Bar (Gauge) with `MM:SS / MM:SS` labels.
   - Chapter list overlay (`Tab`) implemented.

## Instructions for Next Session
1. **Highlight Current Chapter:** In `draw_chapter_list` (screens.rs), use `playback_status.elapsed` to identify the current chapter and apply a "Playing" style (e.g., Bold + Different Color).
2. **MPD Resilience:** Add a heartbeat or reconnection task to `MpdHandler`. Currently, if MPD restarts, the app must be restarted.
3. **Volume Control:** Map `[` and `]` to decrement/increment MPD volume (using `commands::Volume`).
4. **Metadata Polling:** Ensure the chapter list in `App` is kept in sync if MPD automatically moves to the next part/track.
5. **Series/Collections:** Refine `handle_home_events` so that selecting a Series opens a list of its books rather than attempting to "play" the series ID directly.
