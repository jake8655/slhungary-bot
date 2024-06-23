use poise::CreateReply;
use serenity::{
    all::{ChannelId, ChannelType, GuildId},
    async_trait,
};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use tracing::info;

use crate::{
    commands::{Context, Error},
    utils::send_reply,
};

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                info!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}

/// Join your voice channel
#[poise::command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let Some((guild_id, channel_id)) = get_guild_and_channel_id(&ctx).await.ok() else {
        send_reply(
            &ctx,
            CreateReply::default()
                .content("You are not in a voice channel")
                .ephemeral(true),
        )
        .await;
        return Ok(());
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Expected songbird voice client to be placed in at initialization");

    if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    send_reply(&ctx, CreateReply::default().content("Joined voice channel")).await;

    Ok(())
}

async fn get_guild_and_channel_id(ctx: &Context<'_>) -> Result<(GuildId, ChannelId), Error> {
    let guild_id = ctx.guild_id().ok_or("No guild")?;
    let channels = guild_id.channels(&ctx.http()).await?;

    let mut voice_channels = channels.values().filter(|c| c.kind == ChannelType::Voice);

    let result = voice_channels.find_map(|channel| {
        let channel_members = channel.members(ctx.cache()).unwrap();

        channel_members
            .iter()
            .find_map(|m| {
                if m.user.id == ctx.author().id {
                    Some(channel.id)
                } else {
                    None
                }
            })
            .map(|channel_id| (guild_id, channel_id))
    });

    result.ok_or("No voice channel found".into())
}
