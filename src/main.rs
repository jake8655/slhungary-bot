use poise::samples::register_in_guild;
use songbird::SerenityInit;
use std::env;
use std::sync::Arc;
use tracing::error;

use dotenv::dotenv;
use reqwest::Client as HttpClient;
use serenity::all::{GuildId, ShardManager};
use serenity::{all::Color, prelude::*};

mod config;
use config::Config;

mod events;
use events::message::MessageHandler;
use events::ready::ReadyHandler;

mod commands;
use commands::Data;

pub mod utils;

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

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let config_clone = config_mutex.clone();
    let poise_framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::ping(), commands::play(), commands::join()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let guild_id = GuildId::from(config_clone.lock().await.guild_id);
                register_in_guild(&ctx.http, &framework.options().commands, guild_id).await?;

                // For resetting all commands when discord is bugged and has a bunch of old commands registered
                // guild_id.set_commands(&ctx.http, vec![]).await.unwrap();
                // Command::set_global_commands(&ctx.http, vec![])
                //     .await
                //     .unwrap();

                Ok(Data {
                    http_client: HttpClient::new(),
                })
            })
        })
        .build();

    let mut client = Client::builder(&token, intents)
        .event_handler(ReadyHandler)
        .event_handler(MessageHandler)
        .framework(poise_framework)
        .register_songbird()
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
