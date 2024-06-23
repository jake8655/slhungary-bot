use poise::CreateReply;
use songbird::input::YoutubeDl;

use crate::{
    commands::{Context, Error},
    utils::send_reply,
};

/// Play some music
#[poise::command(slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "The URL of the song to play"] url: String,
) -> Result<(), Error> {
    let do_search = !url.starts_with("http");
    let guild_id = ctx.guild_id().unwrap();

    let http_client = ctx.data().http_client.clone();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Expected songbird voice client to be placed in at initialization");

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let src = if do_search {
            YoutubeDl::new_search(http_client, url)
        } else {
            YoutubeDl::new(http_client, url)
        };

        handler.play_input(src.into());
    } else {
        let reply = CreateReply::default()
            .content("Not in a voice channel to play in")
            .ephemeral(true);

        send_reply(&ctx, reply).await;
    }

    send_reply(&ctx, CreateReply::default().content("Playing")).await;

    Ok(())
}
