use reqwest::Client as HttpClient;

pub struct Data {
    pub http_client: HttpClient,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

mod ping;
pub use ping::ping;
mod play;
pub use play::play;
mod leave;
pub use leave::leave;
mod skip;
pub use skip::skip;
mod pause;
pub use pause::pause;
mod resume;
pub use resume::resume;
