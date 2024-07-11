use serenity::all::ChannelId;
use tracing::error;

use serenity::all::Context;
use serenity::all::CreateEmbed;
use serenity::all::CreateEmbedAuthor;
use serenity::all::CreateMessage;
use serenity::all::EventHandler;
use serenity::all::Message;
use serenity::all::ReactionType;
use serenity::all::Timestamp;
use serenity::async_trait;
use tracing::info;

use crate::utils::delete_message;
use crate::utils::react_to_message;
use crate::utils::send_message;
use crate::ClientData;
use crate::BRAND_COLOR;
use crate::BRAND_ICON;
use crate::WARNING_COLOR;

pub struct MessageHandler;

#[async_trait]
impl EventHandler for MessageHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let client_data = ctx.data.read().await;
        let (_, config) = client_data.get::<ClientData>().unwrap();

        if config.read().await.suggestions_channel_id.to_string() == msg.channel_id.to_string() {
            suggestion(&ctx, &msg).await;
        }

        if config.read().await.bug_report_channel_id.to_string() == msg.channel_id.to_string() {
            bug_report(&ctx, &msg).await;
        }
    }
}

async fn suggestion(ctx: &Context, msg: &Message) {
    let client_data = ctx.data.read().await;
    let (_, config) = client_data.get::<ClientData>().unwrap();

    config.write().await.data_json.increment_suggestion_count();

    let embed = CreateMessage::new().embed(
        CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&msg.author.name)
                    .icon_url(msg.author.avatar_url().unwrap_or(BRAND_ICON.to_string())),
            )
            .title(format!(
                "Ötlet - #{}",
                config.read().await.data_json.suggestion_count
            ))
            .description(&msg.content)
            .timestamp(Timestamp::now())
            .color(BRAND_COLOR),
    );

    delete_message(&ctx.http, msg).await;

    match msg.channel_id.send_message(&ctx.http, embed).await {
        Ok(suggestion_msg) => {
            // Add upvote reaction
            react_to_message(
                &ctx.http,
                &suggestion_msg,
                ReactionType::Unicode(String::from("⬆️")),
            )
            .await;
            // Add downvote reaction
            react_to_message(
                &ctx.http,
                &suggestion_msg,
                ReactionType::Unicode(String::from("⬇️")),
            )
            .await;

            // Save suggestion_count to json file
            config.read().await.data_json.save();

            info!(
                "Suggestion message received with id: {}",
                msg.id.to_string()
            );
        }
        Err(e) => {
            error!("Error sending message: {e:?}");
        }
    };
}

async fn bug_report(ctx: &Context, msg: &Message) {
    let client_data = ctx.data.read().await;
    let (_, config) = client_data.get::<ClientData>().unwrap();

    config.write().await.data_json.increment_bug_report_count();

    let user_embed = CreateMessage::new().embed(
        CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&msg.author.name)
                    .icon_url(msg.author.avatar_url().unwrap_or(BRAND_ICON.to_string())),
            )
            .title(format!(
                "Hibajelentés - #{}",
                config.read().await.data_json.bug_report_count
            ))
            .description("Hibajelentésed sikeresen elküldve!")
            .timestamp(Timestamp::now())
            .color(WARNING_COLOR),
    );

    delete_message(&ctx.http, msg).await;

    send_message(&ctx.http, msg.channel_id, user_embed).await;

    let log_channel = ctx
        .http
        .get_channel(ChannelId::new(config.read().await.bug_log_channel_id))
        .await
        .expect("Expected to find the bug log channel")
        .guild()
        .expect("Expected the bug log channel to be in a guild");

    let log_embed = CreateMessage::new().embed(
        CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&msg.author.name)
                    .icon_url(msg.author.avatar_url().unwrap_or(BRAND_ICON.to_string())),
            )
            .title(format!(
                "Hibajelentés - #{}",
                config.read().await.data_json.bug_report_count
            ))
            .description(&msg.content)
            .timestamp(Timestamp::now())
            .color(WARNING_COLOR),
    );

    if let Some(sent_message) = send_message(&ctx.http, log_channel.into(), log_embed).await {
        info!(
            "Bug report received with id: {}",
            sent_message.id.to_string()
        );
    }
}
