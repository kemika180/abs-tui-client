use crate::app::{App, Screen};
use crate::ui::screens;
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn enter(&self) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&self) -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|f| {
            match app.current_screen {
                Screen::Login => screens::draw_login(f, app),
                Screen::Home => screens::draw_home(f, app),
                Screen::Library => screens::draw_library(f, app),
                Screen::Player => screens::draw_player(f, app),
            }
        })?;
        Ok(())
    }

    pub async fn handle_events(&mut self, app: &mut App) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.current_screen {
                    Screen::Login => self.handle_login_events(app, key.code).await?,
                    Screen::Home => self.handle_home_events(app, key).await?,
                    Screen::Library => self.handle_library_events(app, key).await?,
                    Screen::Player => self.handle_player_events(app, key).await?,
                }
            }
        }
        Ok(())
    }

    async fn handle_home_events(&mut self, app: &mut App, key: KeyEvent) -> Result<()> {
        let mut total_items: usize = 0;
        let mut entity_map = Vec::new();

        for view in &app.personalized_views {
            total_items += 1; // Header
            entity_map.push(None);
            for entity in &view.entities {
                total_items += 1;
                entity_map.push(Some(entity));
            }
        }

        match key.code {
            KeyCode::Char('q') => app.quit(),
            KeyCode::Char('l') => {
                app.current_screen = Screen::Library;
                app.selected_index = 0;
                app.library_list_state.select(Some(0));
                app.perform_search().await?;
            }
            KeyCode::Char('p') => {
                app.current_screen = Screen::Player;
            }
            KeyCode::Tab => {
                if !app.libraries.is_empty() {
                    let current_idx = app.libraries.iter().position(|l| Some(&l.id) == app.selected_library_id.as_ref()).unwrap_or(0);
                    let next_idx = (current_idx + 1) % app.libraries.len();
                    app.selected_library_id = Some(app.libraries[next_idx].id.clone());
                    app.fetch_personalized_views().await?;
                    app.selected_index = 0;
                    app.home_list_state.select(Some(0));
                }
            }
            KeyCode::Down | KeyCode::Char('j') if app.settings.vim_motions => {
                if total_items > 0 {
                    app.selected_index = (app.selected_index + 1).min(total_items.saturating_sub(1));
                    app.home_list_state.select(Some(app.selected_index));
                }
            }
            KeyCode::Up | KeyCode::Char('k') if app.settings.vim_motions => {
                app.selected_index = app.selected_index.saturating_sub(1);
                app.home_list_state.select(Some(app.selected_index));
            }
            KeyCode::Enter => {
                let mut target_id = None;
                if let Some(Some(entity)) = entity_map.get(app.selected_index) {
                    target_id = entity.id.clone();
                }
                
                if let Some(id) = target_id {
                    app.play_book(&id).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_library_events(&mut self, app: &mut App, key: KeyEvent) -> Result<()> {
        if app.input_mode {
            match key.code {
                KeyCode::Enter => {
                    app.perform_search().await?;
                    app.input_mode = false;
                }
                KeyCode::Esc => {
                    app.input_mode = false;
                }
                KeyCode::Char(c) => {
                    app.search_query.push(c);
                    app.perform_search().await?;
                }
                KeyCode::Backspace => {
                    app.search_query.pop();
                    app.perform_search().await?;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') => app.quit(),
                KeyCode::Char('h') if app.settings.vim_motions => {
                    app.current_screen = Screen::Home;
                    app.selected_index = 0;
                    app.home_list_state.select(Some(0));
                }
                KeyCode::Char('p') => app.current_screen = Screen::Player,
                KeyCode::Char('i') => {
                    app.input_mode = true;
                }
                KeyCode::Enter => {
                    let mut target_id = None;
                    if let Some(book) = app.search_results.get(app.selected_index) {
                        target_id = book.id.clone();
                    }
                    
                    if let Some(id) = target_id {
                        app.play_book(&id).await?;
                    } else {
                        app.input_mode = true;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') if app.settings.vim_motions => {
                    let max_idx = app.search_results.len().saturating_sub(1);
                    app.selected_index = (app.selected_index + 1).min(max_idx);
                    app.library_list_state.select(Some(app.selected_index));
                }
                KeyCode::Up | KeyCode::Char('k') if app.settings.vim_motions => {
                    app.selected_index = app.selected_index.saturating_sub(1);
                    app.library_list_state.select(Some(app.selected_index));
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_player_events(&mut self, app: &mut App, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => app.quit(),
            KeyCode::Char('h') if app.settings.vim_motions => {
                app.current_screen = Screen::Home;
                app.selected_index = 0;
                app.home_list_state.select(Some(0));
            }
            KeyCode::Char('l') => {
                app.current_screen = Screen::Library;
                app.selected_index = 0;
                app.library_list_state.select(Some(0));
                app.perform_search().await?;
            }
            KeyCode::Char('p') | KeyCode::Char(' ') => {
                if let Some(handler) = &app.mpd_handler {
                    handler.toggle_pause().await?;
                }
            }
            KeyCode::Char(',') => {
                if let Some(handler) = &app.mpd_handler {
                    let duration = Duration::from_secs(app.settings.step_seconds);
                    let _ = handler.client().command(mpd_client::commands::Seek(mpd_client::commands::SeekMode::Backward(duration))).await;
                }
            }
            KeyCode::Char('.') => {
                if let Some(handler) = &app.mpd_handler {
                    let duration = Duration::from_secs(app.settings.step_seconds);
                    let _ = handler.client().command(mpd_client::commands::Seek(mpd_client::commands::SeekMode::Forward(duration))).await;
                }
            }
            KeyCode::Char('<') => {
                let _ = self.navigate_chapter(app, -1).await;
            }
            KeyCode::Char('>') => {
                let _ = self.navigate_chapter(app, 1).await;
            }
            KeyCode::Tab => {
                app.show_chapters = !app.show_chapters;
                app.selected_index = 0;
                app.chapter_list_state.select(Some(0));
            }
            KeyCode::Down | KeyCode::Char('j') if app.settings.vim_motions && app.show_chapters => {
                app.selected_index = (app.selected_index + 1).min(app.current_chapters.len().saturating_sub(1));
                app.chapter_list_state.select(Some(app.selected_index));
            }
            KeyCode::Up | KeyCode::Char('k') if app.settings.vim_motions && app.show_chapters => {
                app.selected_index = app.selected_index.saturating_sub(1);
                app.chapter_list_state.select(Some(app.selected_index));
            }
            KeyCode::Enter if app.show_chapters => {
                if let Some(chapter) = app.current_chapters.get(app.selected_index) {
                    if let Some(handler) = &app.mpd_handler {
                        let _ = handler.seek(chapter.start).await;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn navigate_chapter(&mut self, app: &mut App, delta: i32) -> Result<()> {
        if let Some(handler) = &app.mpd_handler {
            let status = handler.get_status().await?;
            if let Some(elapsed) = status.elapsed {
                let current_time = elapsed.as_secs_f64();
                
                let mut target_chapter_index = None;
                for (i, chapter) in app.current_chapters.iter().enumerate() {
                    if current_time >= chapter.start && current_time < chapter.end {
                        target_chapter_index = Some(i as i32);
                        break;
                    }
                }

                if let Some(idx) = target_chapter_index {
                    let mut new_idx = idx + delta;
                    
                    if delta < 0 && current_time > app.current_chapters[idx as usize].start + 2.0 {
                        new_idx = idx;
                    }

                    if new_idx >= 0 && (new_idx as usize) < app.current_chapters.len() {
                        handler.seek(app.current_chapters[new_idx as usize].start).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_login_events(&mut self, app: &mut App, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Tab => {
                app.login_form.cursor_index = (app.login_form.cursor_index + 1) % 3;
            }
            KeyCode::Enter => {
                app.login().await?;
            }
            KeyCode::Char(c) => {
                let target = match app.login_form.cursor_index {
                    0 => &mut app.login_form.url,
                    1 => &mut app.login_form.username,
                    2 => &mut app.login_form.password,
                    _ => unreachable!(),
                };
                target.push(c);
            }
            KeyCode::Backspace => {
                let target = match app.login_form.cursor_index {
                    0 => &mut app.login_form.url,
                    1 => &mut app.login_form.username,
                    2 => &mut app.login_form.password,
                    _ => unreachable!(),
                };
                target.pop();
            }
            _ => {}
        }
        Ok(())
    }
}
