use serenity::all::Ready;
use serenity::async_trait;
use serenity::prelude::*;
use tracing::info;

pub struct ReadyHandler;

#[async_trait]
impl EventHandler for ReadyHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is online!", ready.user.name);
    }
}
