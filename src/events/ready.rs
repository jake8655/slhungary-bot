use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use serenity::all::ChannelId;
use serenity::all::CreateActionRow;
use serenity::all::CreateButton;
use serenity::all::CreateEmbed;
use serenity::all::CreateEmbedFooter;
use serenity::all::CreateMessage;
use serenity::all::EditMessage;
use serenity::all::Ready;
use serenity::all::Timestamp;
use serenity::async_trait;
use serenity::prelude::*;
use tokio::task;
use tokio::time;
use tracing::error;
use tracing::info;

use crate::ClientData;
use crate::BRAND_COLOR;
use crate::BRAND_NAME;
use crate::BRAND_NAME_SHORT;
use crate::BRAND_WEBSITE;
use crate::ERROR_COLOR;

pub struct ReadyHandler;

#[async_trait]
impl EventHandler for ReadyHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is online!", ready.user.name);

        // Set activity to Do Not Disturb
        ctx.dnd();

        manage_server_status_message(ctx).await;
    }
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

async fn manage_server_status_message(ctx: Context) {
    let client_data = ctx.data.read().await;
    let (_, config) = client_data.get::<ClientData>().unwrap();

    let embed = CreateEmbed::new()
        .title(format!("{} | Szerver Státusz", BRAND_NAME))
        .description(format!("A <#{}> csatornában mindig értesülsz a szerver aktuális elérhetőségéről és állapotáról!", config.lock().await.status_channel_id))
        .footer(CreateEmbedFooter::new(BRAND_NAME_SHORT))
        .timestamp(Timestamp::now())
        .color(BRAND_COLOR);

    drop(client_data);

    let context_arc = Arc::new(ctx);

    let forever = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;
            send_or_edit_message(context_arc.clone(), embed.clone()).await;
        }
    });

    forever.await.expect("Expected to spawn task");
}

async fn send_or_edit_message(ctx: Arc<Context>, mut embed: CreateEmbed) {
    let client_data = ctx.data.read().await;
    let (_, config) = client_data.get::<ClientData>().unwrap();

    let response = get_players(&ctx).await;

    match response {
        Ok((players, server_info)) => {
            embed = embed.fields(vec![
                ("Szerver Státusz:", "✅ Elérhető", true),
                (
                    "Elérhető Játékosok:",
                    &format!("{}/{}", players.len(), server_info.vars.max_players),
                    true,
                ),
                ("Következő Újraindításig:", "6 óra 15 perc", true),
            ]);

            if players.len() > 0 {
                let mut player_values = [
                    String::from("**Játékosok:**\n"),
                    String::from(""),
                    String::from(""),
                ];

                for (i, player) in players.iter().enumerate() {
                    player_values[(i + 1) % 3] +=
                        &format!("{} ({}ms)\n", truncate(&player.name, 12), player.ping);
                }

                let player_fields = player_values
                    .iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| ("\u{200b}", s, true));

                embed = embed.fields(player_fields);
            }
        }
        Err(e) => {
            error!("Error getting players: {e:?}");

            embed = embed
                .fields(vec![("Szerver Státusz:", "❌ Nem elérhető", true)])
                .color(ERROR_COLOR);
        }
    }

    let mut locked_config = config.lock().await;

    let channel = ctx
        .http
        .get_channel(ChannelId::new(locked_config.status_channel_id))
        .await
        .expect("Expected to find the status channel")
        .guild()
        .expect("Expected the status channel to be in a guild");

    match locked_config.data_json.status_message_id {
        Some(id) => {
            let message =
                EditMessage::new()
                    .embed(embed)
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new_link(BRAND_WEBSITE).label("Weboldal"),
                        CreateButton::new_link(format!(
                            "https://discord.com/channels/{}/{}",
                            locked_config.guild_id, locked_config.help_channel_id
                        ))
                        .label("Segítségkérés"),
                    ])]);

            match channel.edit_message(&ctx.http, id, message).await {
                Ok(message) => info!("Edited status message with id: {}", message.id.to_string()),
                Err(e) => error!("Error editing message: {e:?}"),
            }
        }
        None => {
            let message =
                CreateMessage::new()
                    .embed(embed)
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new_link(BRAND_WEBSITE).label("Weboldal"),
                        CreateButton::new_link(format!(
                            "https://discord.com/channels/{}/{}",
                            locked_config.guild_id, locked_config.help_channel_id
                        ))
                        .label("Segítségkérés"),
                    ])]);

            match channel.send_message(&ctx.http, message).await {
                Ok(message) => {
                    locked_config
                        .data_json
                        .set_status_message_id(message.id.to_string().parse::<u64>().unwrap());
                    locked_config.data_json.save();

                    info!(
                        "Created new status message with id: {}",
                        message.id.to_string()
                    );
                }
                Err(e) => error!("Error sending message: {e:?}"),
            }
        }
    };
}

#[derive(Deserialize)]
struct Player {
    name: String,
    ping: u32,
}

#[derive(Deserialize)]
struct ServerInfo {
    vars: Vars,
}

#[derive(Deserialize)]
struct Vars {
    #[serde(rename = "sv_maxClients")]
    max_players: String,
}

async fn get_players(ctx: &Context) -> Result<(Box<[Player]>, ServerInfo)> {
    let client_data = ctx.data.read().await;
    let (_, config) = client_data.get::<ClientData>().unwrap();

    let fivem_ip = &config.lock().await.fivem_ip;

    let players = reqwest::get(format!("{}/players.json", fivem_ip))
        .await?
        .json::<Box<[Player]>>()
        .await?;

    let server_info = reqwest::get(format!("{}/info.json", fivem_ip))
        .await?
        .json::<ServerInfo>()
        .await?;

    Ok((players, server_info))
}
