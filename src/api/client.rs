use crate::api::models::{PersonalizedView, Book, PlaybackSession, Library};
use color_eyre::eyre::{Result, eyre};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;

pub struct ApiClient {
    url: String,
    token: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(url: String, token: String) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            token,
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", self.token)) {
            headers.insert(AUTHORIZATION, val);
        }
        headers
    }

    pub async fn get_libraries(&self) -> Result<Vec<Library>> {
        let url = format!("{}/api/libraries", self.url);
        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(eyre!("Failed to fetch libraries: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct LibrariesResponse {
            libraries: Vec<Library>,
        }

        let res: LibrariesResponse = response.json().await?;
        Ok(res.libraries)
    }

    pub async fn get_personalized_view(&self, library_id: &str) -> Result<Vec<PersonalizedView>> {
        let url = format!("{}/api/libraries/{}/personalized", self.url, library_id);
        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(eyre!("Failed to fetch personalized view: {}", response.status()));
        }

        let views: Vec<PersonalizedView> = response.json().await?;
        Ok(views)
    }

    pub async fn get_library_items(&self, library_id: &str) -> Result<Vec<Book>> {
        let url = format!("{}/api/libraries/{}/items?limit=0", self.url, library_id);
        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(eyre!("Failed to fetch library items: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct LibraryItemsResponse {
            results: Vec<Book>,
        }

        let res: LibraryItemsResponse = response.json().await?;
        Ok(res.results)
    }

    pub async fn search(&self, _library_id: &str, _query: &str) -> Result<Vec<Book>> {
        // This is now handled locally in the app, but keeping the signature for now
        Ok(Vec::new())
    }

    pub async fn start_playback_session(&self, item_id: &str) -> Result<PlaybackSession> {
        let url = format!("{}/api/items/{}/play", self.url, item_id);
        let params = serde_json::json!({
            "deviceInfo": {
                "clientName": "abs-tui-client",
                "deviceType": "tui"
            },
            "forceDirectPlay": true
        });

        let response = self.client
            .post(&url)
            .headers(self.headers())
            .json(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(eyre!("Failed to start playback session: {}", response.status()));
        }

        let session: PlaybackSession = response.json().await?;
        Ok(session)
    }

    pub async fn get_item_details(&self, item_id: &str) -> Result<Book> {
        let url = format!("{}/api/items/{}", self.url, item_id);
        let response = self.client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(eyre!("Failed to get item details: {}", response.status()));
        }

        let book: Book = response.json().await?;
        Ok(book)
    }
}
