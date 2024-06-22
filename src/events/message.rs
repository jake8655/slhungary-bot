use serenity::all::ChannelId;
use serenity::all::Color;
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

        if config.lock().await.suggestions_channel_id.to_string() == msg.channel_id.to_string() {
            suggestion(&ctx, &msg).await;
        }

        if config.lock().await.bug_report_channel_id.to_string() == msg.channel_id.to_string() {
            bug_report(&ctx, &msg).await;
        }
    }
}

async fn suggestion(ctx: &Context, msg: &Message) {
    let client_data = ctx.data.read().await;
    let (_, config) = client_data.get::<ClientData>().unwrap();

    config.lock().await.data_json.increment_suggestion_count();

    let embed = CreateMessage::new().embed(
        CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&msg.author.name)
                    .icon_url(msg.author.avatar_url().unwrap_or(BRAND_ICON.to_string())),
            )
            .title(format!(
                "Ötlet - #{}",
                config.lock().await.data_json.suggestion_count
            ))
            .description(&msg.content)
            .timestamp(Timestamp::now())
            .color(BRAND_COLOR),
    );

    if let Err(e) = msg.delete(&ctx.http).await {
        error!("Error deleting message: {e:?}");
    }

    match msg.channel_id.send_message(&ctx.http, embed).await {
        Ok(suggestion_msg) => {
            // Add upvote reaction
            if let Err(e) = suggestion_msg
                .react(&ctx.http, ReactionType::Unicode(String::from("⬆️")))
                .await
            {
                error!("Error reacting to message: {e:?}");
            }
            // Add downvote reaction
            if let Err(e) = suggestion_msg
                .react(&ctx.http, ReactionType::Unicode(String::from("⬇️")))
                .await
            {
                error!("Error reacting to message: {e:?}");
            }

            // Save suggestion_count to json file
            config.lock().await.data_json.save();
        }
        Err(e) => {
            error!("Error sending message: {e:?}");
        }
    };
}

async fn bug_report(ctx: &Context, msg: &Message) {
    let client_data = ctx.data.read().await;
    let (_, config) = client_data.get::<ClientData>().unwrap();

    config.lock().await.data_json.increment_bug_report_count();

    let user_embed = CreateMessage::new().embed(
        CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&msg.author.name)
                    .icon_url(msg.author.avatar_url().unwrap_or(BRAND_ICON.to_string())),
            )
            .title(format!(
                "Hibajelentés - #{}",
                config.lock().await.data_json.bug_report_count
            ))
            .description("Hibajelentésed sikeresen elküldve!")
            .timestamp(Timestamp::now())
            .color(Color::DARK_RED),
    );

    if let Err(e) = msg.delete(&ctx.http).await {
        error!("Error deleting message: {e:?}");
    }

    if let Err(e) = msg.channel_id.send_message(&ctx.http, user_embed).await {
        error!("Error sending message: {e:?}");
    };

    let log_channel = ctx
        .http
        .get_channel(ChannelId::new(config.lock().await.bug_log_channel_id))
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
                config.lock().await.data_json.bug_report_count
            ))
            .description(&msg.content)
            .timestamp(Timestamp::now())
            .color(WARNING_COLOR),
    );

    if let Err(e) = log_channel.send_message(&ctx.http, log_embed).await {
        error!("Error sending message: {e:?}");
    };
}
