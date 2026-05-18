pub mod api;
pub mod config;
pub mod player;
pub mod ui;
pub mod app;

use crate::app::{App, Screen};
use crate::config::settings::load_settings;
use crate::ui::tui_handler::Tui;
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let settings = load_settings()?;
    let mut app = App::new(settings);
    
    if let Ok(handler) = crate::player::mpd_handler::MpdHandler::new(&app.settings.mpd.address).await {
        app.mpd_handler = Some(handler);
    }

    let mut tui = Tui::new()?;

    tui.enter()?;

    if matches!(app.current_screen, Screen::Home) {
        let _ = app.fetch_personalized_views().await;
    }

    while !app.should_quit {
        tui.draw(&mut app)?;
        tui.handle_events(&mut app).await?;
        app.poll_status().await?;
    }

    tui.exit()?;

    Ok(())
}
