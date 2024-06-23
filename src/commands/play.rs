use poise::CreateReply;
use serenity::{
    all::{ChannelId, ChannelType, GuildId},
    async_trait,
};
use songbird::{
    input::YoutubeDl, Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};
use tracing::{error, info};

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

/// Play some music
#[poise::command(slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "The URL of the song to play"] url: String,
) -> Result<(), Error> {
    let do_search = !url.starts_with("http");
    let Some((guild_id, channel_id)) = get_guild_and_channel_id(&ctx).await else {
        send_reply(
            &ctx,
            CreateReply::default()
                .content("You are not in a voice channel")
                .ephemeral(true),
        )
        .await;
        return Ok(());
    };

    let http_client = ctx.data().http_client.clone();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Expected songbird voice client to be placed in at initialization");

    match manager.join(guild_id, channel_id).await {
        Ok(handler_lock) => {
            let mut handler = handler_lock.lock().await;

            handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
            handler.deafen(true).await.ok();

            let src = if do_search {
                YoutubeDl::new_search(http_client, url)
            } else {
                YoutubeDl::new(http_client, url)
            };

            send_reply(
                &ctx,
                CreateReply::default().content(format!(
                    "Added song to queue: position {}",
                    handler.queue().len() + 1
                )),
            )
            .await;

            handler.enqueue_input(src.into()).await;

            Ok(())
        }
        Err(e) => {
            error!("Failed to join voice channel: {e:?}");
            Ok(())
        }
    }
}

async fn get_guild_and_channel_id(ctx: &Context<'_>) -> Option<(GuildId, ChannelId)> {
    let guild_id = ctx.guild_id()?;
    let channels = guild_id.channels(&ctx.http()).await.ok()?;

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

    result
}
