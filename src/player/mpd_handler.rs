use mpd_client::{Client, commands};
use tokio::net::TcpStream;
use color_eyre::eyre::Result;

pub struct MpdHandler {
    client: Client,
}

impl MpdHandler {
    pub async fn new(address: &str) -> Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let (client, _) = Client::connect(stream).await?;
        Ok(Self { client })
    }

    pub async fn play(&self) -> Result<()> {
        self.client.command(commands::Play::current()).await?;
        Ok(())
    }

    pub async fn pause(&self, pause: bool) -> Result<()> {
        self.client.command(commands::SetPause(pause)).await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.client.command(commands::Stop).await?;
        Ok(())
    }

    pub async fn clear_queue(&self) -> Result<()> {
        self.client.command(commands::ClearQueue).await?;
        Ok(())
    }

    pub async fn add_to_queue(&self, url: &str) -> Result<()> {
        self.client.command(commands::Add::uri(url)).await?;
        Ok(())
    }

    pub async fn play_pos(&self, pos: usize) -> Result<()> {
        self.client.command(commands::Play::song(commands::SongPosition(pos))).await?;
        Ok(())
    }

    pub async fn get_status(&self) -> Result<mpd_client::responses::Status> {
        let status = self.client.command(commands::Status).await?;
        Ok(status)
    }

    pub async fn seek(&self, seconds: f64) -> Result<()> {
        let duration = std::time::Duration::from_secs_f64(seconds);
        self.client.command(commands::Seek(commands::SeekMode::Absolute(duration))).await?;
        Ok(())
    }

    pub async fn toggle_pause(&self) -> Result<()> {
        let status = self.get_status().await?;
        let is_paused = matches!(status.state, mpd_client::responses::PlayState::Paused);
        let is_playing = matches!(status.state, mpd_client::responses::PlayState::Playing);
        
        if is_playing {
            self.pause(true).await?;
        } else if is_paused {
            self.pause(false).await?;
        } else {
            self.play().await?;
        }
        Ok(())
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
