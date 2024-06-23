use poise::CreateReply;

use crate::{
    commands::{Context, Error},
    utils::send_reply,
};

#[poise::command(slash_command)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Expected songbird voice client to be placed in at initialization");

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        queue.skip().ok();

        send_reply(
            &ctx,
            CreateReply::default().content(format!("Song skipped: {} in queue", queue.len())),
        )
        .await;
    } else {
        send_reply(
            &ctx,
            CreateReply::default()
                .content("Not in a voice channel to play in")
                .ephemeral(true),
        )
        .await;
    }

    Ok(())
}
