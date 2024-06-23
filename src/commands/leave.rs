use poise::CreateReply;

use crate::{
    commands::{Context, Error},
    utils::send_reply,
};

/// Leave your voice channel
#[poise::command(slash_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Expected songbird voice client to be placed in at initialization");

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        match manager.remove(guild_id).await {
            Ok(_) => {
                send_reply(&ctx, CreateReply::default().content("Left voice channel")).await;
            }
            Err(e) => {
                send_reply(
                    &ctx,
                    CreateReply::default()
                        .content(format!("Error: {e}"))
                        .ephemeral(true),
                )
                .await;
            }
        }
    } else {
        send_reply(
            &ctx,
            CreateReply::default()
                .content("Not in a voice channel")
                .ephemeral(true),
        )
        .await;
    }

    Ok(())
}
