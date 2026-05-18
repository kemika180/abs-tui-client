use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Gauge, List, ListItem, Clear},
    Frame,
};

pub fn draw_login(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.area());

    let url_input = Paragraph::new(app.login_form.url.as_str())
        .block(Block::default().borders(Borders::ALL).title("Server URL"))
        .style(if app.login_form.cursor_index == 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    f.render_widget(url_input, chunks[0]);

    let username_input = Paragraph::new(app.login_form.username.as_str())
        .block(Block::default().borders(Borders::ALL).title("Username"))
        .style(if app.login_form.cursor_index == 1 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    f.render_widget(username_input, chunks[1]);

    let password_input = Paragraph::new("*".repeat(app.login_form.password.len()))
        .block(Block::default().borders(Borders::ALL).title("Password"))
        .style(if app.login_form.cursor_index == 2 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    f.render_widget(password_input, chunks[2]);

    let help = Paragraph::new("Tab to switch fields, Enter to login, 'q' to quit")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(help, chunks[3]);

    if let Some(error) = &app.error_message {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        let area = centered_rect(50, 20, f.area());
        f.render_widget(Clear, area);
        f.render_widget(error_msg, area);
    }
}

pub fn draw_home(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    let lib_name = app.libraries.iter()
        .find(|l| Some(&l.id) == app.selected_library_id.as_ref())
        .map(|l| l.name.as_str())
        .unwrap_or("Home");

    let header = Paragraph::new(format!("Audiobookshelf TUI - {}", lib_name))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    if let Some(error) = &app.error_message {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(error_msg, chunks[1]);
        return;
    }

    if app.libraries.is_empty() {
        let msg = Paragraph::new("No libraries found on server.")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(msg, chunks[1]);
        return;
    }

    if app.personalized_views.is_empty() {
        let msg = Paragraph::new("No content available for this library.\nTry another library or add some books!")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(msg, chunks[1]);
        return;
    }

    let mut final_items = Vec::new();
    for view in &app.personalized_views {
        final_items.push(ListItem::new(format!("--- {} ---", view.label)).style(Style::default().fg(Color::Yellow)));
        
        for entity in &view.entities {
            let name = if let Some(name) = &entity.name {
                name.clone()
            } else if let Some(media) = &entity.media {
                media.metadata.as_ref()
                    .and_then(|m| m.title.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };

            final_items.push(ListItem::new(format!("  {}", name)));
        }
    }

    let body = List::new(final_items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Rgb(51, 71, 110)).fg(Color::White))
        .highlight_symbol("➤ ");
    
    f.render_stateful_widget(body, chunks[1], &mut app.home_list_state);

    let footer = Paragraph::new("'q' to quit, 'l' for library, 'p' for player, Tab to switch libraries, hjkl to navigate")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[2]);
}

pub fn draw_library(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    let (search_title, search_style) = if app.input_mode {
        ("Search Library [INSERT]", Style::default().fg(Color::Yellow))
    } else {
        ("Search Library [NORMAL]", Style::default())
    };

    let search_input = Paragraph::new(app.search_query.as_str())
        .block(Block::default().borders(Borders::ALL).title(search_title))
        .style(search_style);
    f.render_widget(search_input, chunks[0]);

    if let Some(error) = &app.error_message {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(error_msg, chunks[1]);
        return;
    }

    if app.all_library_items.is_empty() {
        let msg = Paragraph::new("Loading library items...")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(msg, chunks[1]);
    } else if app.search_results.is_empty() && !app.search_query.is_empty() {
        let msg = Paragraph::new("No results found.")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(msg, chunks[1]);
    } else {
        let display_items = if app.search_query.is_empty() { &app.all_library_items } else { &app.search_results };
        
        let items: Vec<ListItem> = display_items.iter().map(|book| {
            let mut title = "Unknown Title";
            let mut author = "Unknown Author";
            
            if let Some(media) = &book.media {
                if let Some(metadata) = &media.metadata {
                    if let Some(t) = &metadata.title {
                        title = t;
                    }
                    if let Some(a) = &metadata.author_name {
                        author = a;
                    }
                }
            }
            
            ListItem::new(format!("{} - {}", title, author))
        }).collect();

        let results = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Library Items"))
            .highlight_style(Style::default().bg(Color::Rgb(51, 71, 110)).fg(Color::White))
            .highlight_symbol("➤ ");
            
        f.render_stateful_widget(results, chunks[1], &mut app.library_list_state);
    }

    let footer_text = if app.input_mode {
        "Enter to search, Esc for Normal mode"
    } else {
        "'q' to quit, 'h' for home, 'p' for player, 'i'/Enter for Insert mode, hjkl to navigate"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[2]);
}

pub fn draw_player(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1), // Header
                Constraint::Min(0),    // Book Info
                Constraint::Length(1), // Chapter Title/Number
                Constraint::Length(3), // Progress Bar
                Constraint::Length(1), // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    let header = Paragraph::new("Audiobookshelf TUI - Player")
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(header, chunks[0]);

    let player_info_text = if let Some(session) = &app.current_session {
        let title = session.display_title.as_deref().unwrap_or("Unknown Title");
        let author = session.display_author.as_deref().unwrap_or("Unknown Author");
        format!("Now Playing:\n\nTitle: {}\nAuthor: {}", title, author)
    } else {
        "Now Playing:\n\n[No Book Selected]".to_string()
    };

    let player_info = Paragraph::new(player_info_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(player_info, chunks[1]);

    let mut percent = 0;
    let mut progress_label = "00:00 / 00:00".to_string();
    let mut chapter_info = String::from("Progress: [No Chapter Info]");

    if let Some(status) = &app.playback_status {
        if let Some(elapsed) = status.elapsed {
            let current_time = elapsed.as_secs_f64();
            
            // Find current chapter
            let current_chapter = app.current_chapters.iter().enumerate().find(|(_, c)| {
                current_time >= c.start && current_time < c.end
            });

            if let Some((idx, chapter)) = current_chapter {
                chapter_info = format!(
                    "Progress: {}/{} - {}",
                    idx + 1,
                    app.current_chapters.len(),
                    chapter.title
                );

                let chapter_elapsed = current_time - chapter.start;
                let chapter_duration = chapter.end - chapter.start;
                if chapter_duration > 0.0 {
                    percent = (chapter_elapsed / chapter_duration * 100.0) as u16;
                    progress_label = format!(
                        "{:02}:{:02} / {:02}:{:02}",
                        chapter_elapsed as u64 / 60, chapter_elapsed as u64 % 60,
                        chapter_duration as u64 / 60, chapter_duration as u64 % 60
                    );
                }
            } else if let Some(duration) = status.duration {
                // Fallback to full book progress if no chapters or outside chapter ranges
                percent = (current_time / duration.as_secs_f64() * 100.0) as u16;
                progress_label = format!(
                    "{:02}:{:02} / {:02}:{:02}",
                    elapsed.as_secs() / 60, elapsed.as_secs() % 60,
                    duration.as_secs() / 60, duration.as_secs() % 60
                );
            }
        }
    }

    let chapter_para = Paragraph::new(chapter_info)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(chapter_para, chunks[2]);

    let progress = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Chapter Progress"))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(percent.min(100))
        .label(progress_label);
    f.render_widget(progress, chunks[3]);

    let footer = Paragraph::new("'q' to quit, 'h' for home, 'l' for library, Space/p to play/pause, ,/. to seek, </> for chapters, Tab for list")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[4]);

    if app.show_chapters {
        draw_chapter_list(f, app);
    }
}

fn draw_chapter_list(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = app.current_chapters.iter().map(|chapter| {
        ListItem::new(chapter.title.clone())
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Chapters"))
        .highlight_style(Style::default().bg(Color::Rgb(51, 71, 110)).fg(Color::White))
        .highlight_symbol("> ");
    
    f.render_stateful_widget(list, area, &mut app.chapter_list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
