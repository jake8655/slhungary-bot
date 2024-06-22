use std::env;
use std::sync::Arc;
use tracing::error;

use dotenv::dotenv;
use serenity::all::ShardManager;
use serenity::{all::Color, prelude::*};

mod config;
use config::Config;

mod events;
use events::message::MessageHandler;
use events::ready::ReadyHandler;

mod commands;
use commands::ping;
use commands::Data;

pub struct ClientData {}

impl TypeMapKey for ClientData {
    type Value = (Arc<ShardManager>, Arc<Mutex<Config>>);
}

pub const BRAND_COLOR: Color = Color::from_rgb(33, 121, 227);
pub const WARNING_COLOR: Color = Color::from_rgb(172, 20, 20);
pub const ERROR_COLOR: Color = Color::from_rgb(227, 46, 36);
pub const SUCCESS_COLOR: Color = Color::from_rgb(49, 214, 69);
pub const BRAND_ICON: &str =
    "https://cdn.discordapp.com/attachments/766990340212785162/1253416891096240168/slhungary.png";
pub const BRAND_NAME: &str = "Smoke Life RolePlay";
pub const BRAND_NAME_SHORT: &str = "SLRP";
pub const BRAND_WEBSITE: &str = "https://slhungary.com";

#[tokio::main]
async fn main() {
    dotenv().expect("Expected to load `.env` file");

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected `DISCORD_TOKEN` in the environment");

    let config = Config::new();
    let config_mutex = Arc::new(Mutex::new(config));

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let poise_framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = Client::builder(&token, intents)
        .event_handler(ReadyHandler)
        .event_handler(MessageHandler)
        .framework(poise_framework)
        .await
        .expect("Expected to create client");

    {
        let mut data = client.data.write().await;
        data.insert::<ClientData>((client.shard_manager.clone(), config_mutex.clone()));
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Expected to listen for ctrl-c");
        shard_manager.shutdown_all().await;
    });

    if let Err(e) = client.start().await {
        error!("Client error: {e:?}");
    }
}
