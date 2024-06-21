use poise::CreateReply;
use serenity::all::CreateEmbed;

use crate::BRAND_COLOR;

pub struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

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
