use poise::CreateReply;
use serenity::all::CreateEmbed;

use crate::{
    commands::{Context, Error},
    BRAND_COLOR,
};

/// Ping command
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response = CreateReply::default()
        .embed(
            CreateEmbed::new()
                .title("Ping!")
                .description("I am alive bro!")
                .color(BRAND_COLOR),
        )
        .ephemeral(true);

    ctx.send(response).await?;

    Ok(())
}
