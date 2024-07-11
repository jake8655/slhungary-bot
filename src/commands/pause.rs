use poise::CreateReply;
use tracing::error;

use crate::{
    commands::{Context, Error},
    utils::send_reply,
};

/// Pause the current track
#[poise::command(slash_command)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Expected songbird voice client to be placed in at initialization");

    let guild_id = ctx.guild_id().expect("Expected to be in a guild");

    match manager.get(guild_id) {
        None => {
            send_reply(
                &ctx,
                CreateReply::default().content("Not in a voice channel"),
            )
            .await;

            Ok(())
        }
        Some(handler_lock) => {
            let handler = handler_lock.lock().await;
            let queue = handler.queue();

            if let Err(e) = queue.pause() {
                error!("Failed to pause queue: {e:?}");
            };

            send_reply(
                &ctx,
                CreateReply::default().content("Paused playing the track"),
            )
            .await;

            Ok(())
        }
    }
}
