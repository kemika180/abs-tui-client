use serde::{Deserialize, Serialize};
use color_eyre::eyre::{Result, eyre};

#[derive(Debug, Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    user: User,
}

#[derive(Debug, Deserialize)]
struct User {
    token: String,
}

pub async fn login(url: &str, username: &str, password: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let login_url = format!("{}/login", url.trim_end_matches('/'));
    
    let response = client
        .post(&login_url)
        .json(&LoginRequest { username, password })
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(eyre!("Login failed with status: {}", response.status()));
    }

    let login_res: LoginResponse = response.json().await?;
    Ok(login_res.user.token)
}
