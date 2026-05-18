use crate::config::settings::Settings;
use crate::api::client::ApiClient;
use crate::api::models::{PersonalizedView, Book, Chapter, Library, PlaybackSession};
use crate::player::mpd_handler::MpdHandler;
use ratatui::widgets::ListState;

pub enum Screen {
    Login,
    Home,
    Library,
    Player,
}

pub struct App {
    pub settings: Settings,
    pub api_client: Option<ApiClient>,
    pub mpd_handler: Option<MpdHandler>,
    pub current_screen: Screen,
    pub login_form: LoginForm,
    pub should_quit: bool,
    pub libraries: Vec<Library>,
    pub selected_library_id: Option<String>,
    pub personalized_views: Vec<PersonalizedView>,
    pub search_query: String,
    pub search_results: Vec<Book>,
    pub all_library_items: Vec<Book>,
    pub selected_index: usize,
    pub show_chapters: bool,
    pub current_chapters: Vec<Chapter>,
    pub error_message: Option<String>,
    pub input_mode: bool,
    pub current_session: Option<PlaybackSession>,
    pub playback_status: Option<mpd_client::responses::Status>,
    
    // List states for scrolling
    pub home_list_state: ListState,
    pub library_list_state: ListState,
    pub chapter_list_state: ListState,
}

pub struct LoginForm {
    pub url: String,
    pub username: String,
    pub password: String,
    pub cursor_index: usize, // 0: URL, 1: Username, 2: Password
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let (current_screen, api_client) = if let (Some(url), Some(token)) = (settings.server.url.as_ref(), settings.server.token.as_ref()) {
            (Screen::Home, Some(ApiClient::new(url.clone(), token.clone())))
        } else {
            (Screen::Login, None)
        };

        let mut home_list_state = ListState::default();
        home_list_state.select(Some(0));
        let mut library_list_state = ListState::default();
        library_list_state.select(Some(0));
        let mut chapter_list_state = ListState::default();
        chapter_list_state.select(Some(0));

        Self {
            settings,
            api_client,
            mpd_handler: None,
            current_screen,
            login_form: LoginForm {
                url: String::new(),
                username: String::new(),
                password: String::new(),
                cursor_index: 0,
            },
            should_quit: false,
            libraries: Vec::new(),
            selected_library_id: None,
            personalized_views: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            all_library_items: Vec::new(),
            selected_index: 0,
            show_chapters: false,
            current_chapters: Vec::new(),
            error_message: None,
            input_mode: false,
            current_session: None,
            playback_status: None,
            home_list_state,
            library_list_state,
            chapter_list_state,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub async fn fetch_personalized_views(&mut self) -> color_eyre::Result<()> {
        if let Some(client) = &self.api_client {
            // First, ensure we have libraries
            if self.libraries.is_empty() {
                match client.get_libraries().await {
                    Ok(libs) => {
                        self.libraries = libs;
                        if self.selected_library_id.is_none() {
                            self.selected_library_id = self.libraries.first().map(|l| l.id.clone());
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to fetch libraries: {}", e));
                        return Ok(());
                    }
                }
            }

            let lib_id = match &self.selected_library_id {
                Some(id) => id.clone(),
                None => {
                    self.error_message = Some("No libraries found".to_string());
                    return Ok(());
                }
            };

            // Prefetch ALL items for search
            match client.get_library_items(&lib_id).await {
                Ok(items) => {
                    self.all_library_items = items;
                    // Initial search results = all items
                    self.search_results = self.all_library_items.clone();
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to prefetch items: {}", e));
                }
            }

            match client.get_personalized_view(&lib_id).await {
                Ok(views) => {
                    self.personalized_views = views;
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to fetch views: {}", e));
                }
            }
        }
        Ok(())
    }

    pub async fn perform_search(&mut self) -> color_eyre::Result<()> {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.search_results = self.all_library_items.clone();
        } else {
            self.search_results = self.all_library_items.iter()
                .filter(|book| {
                    if let Some(media) = &book.media {
                        if let Some(metadata) = &media.metadata {
                            let title = metadata.title.as_deref().unwrap_or("").to_lowercase();
                            let author = metadata.author_name.as_deref().unwrap_or("").to_lowercase();
                            if title.contains(&query) || author.contains(&query) {
                                return true;
                            }
                        }
                    }
                    false
                })
                .cloned()
                .collect();
        }
        self.selected_index = 0;
        self.library_list_state.select(Some(0));
        Ok(())
    }

    pub async fn login(&mut self) -> color_eyre::Result<()> {
        match crate::api::auth::login(
            &self.login_form.url,
            &self.login_form.username,
            &self.login_form.password,
        ).await {
            Ok(token) => {
                self.settings.server.url = Some(self.login_form.url.clone());
                self.settings.server.username = Some(self.login_form.username.clone());
                self.settings.server.token = Some(token.clone());

                self.api_client = Some(crate::api::client::ApiClient::new(
                    self.login_form.url.clone(),
                    token,
                ));

                crate::config::settings::save_settings(&self.settings)?;
                self.fetch_personalized_views().await?;
                self.current_screen = Screen::Home;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Login failed: {}", e));
            }
        }
        Ok(())
    }

    pub async fn play_book(&mut self, book_id: &str) -> color_eyre::Result<()> {
        if let (Some(api), Some(mpd)) = (&self.api_client, &self.mpd_handler) {
            match api.get_item_details(book_id).await {
                Ok(book) => {
                    self.current_chapters = book.media.and_then(|m| m.chapters).unwrap_or_default();
                    self.chapter_list_state.select(Some(0));
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to fetch chapters: {}", e));
                }
            }

            match api.start_playback_session(book_id).await {
                Ok(session) => {
                    self.current_session = Some(session.clone());
                    
                    if let Err(e) = mpd.clear_queue().await {
                        self.error_message = Some(format!("Failed to clear queue: {}", e));
                        return Ok(());
                    }
                    
                    for track in &session.audio_tracks {
                        let full_url = if track.content_url.starts_with("http") {
                            track.content_url.clone()
                        } else {
                            format!("{}{}?token={}", 
                                self.settings.server.url.as_deref().unwrap_or("").trim_end_matches('/'),
                                track.content_url,
                                self.settings.server.token.as_deref().unwrap_or("")
                            )
                        };
                        if let Err(e) = mpd.add_to_queue(&full_url).await {
                            self.error_message = Some(format!("Failed to add to queue: {}", e));
                            return Ok(());
                        }
                    }

                    let _ = mpd.play().await;
                    if session.current_time > 0.0 {
                        let _ = mpd.seek(session.current_time).await;
                    }
                    
                    self.current_screen = Screen::Player;
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to start playback: {}", e));
                }
            }
        } else {
            self.error_message = Some("API or MPD not initialized".to_string());
        }
        Ok(())
    }

    pub async fn poll_status(&mut self) -> color_eyre::Result<()> {
        if let Some(mpd) = &self.mpd_handler {
            match mpd.get_status().await {
                Ok(status) => {
                    self.playback_status = Some(status);
                }
                Err(_) => {
                    self.playback_status = None;
                }
            }
        }
        Ok(())
    }
}
