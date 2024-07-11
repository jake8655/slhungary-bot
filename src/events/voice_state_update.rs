use serenity::{
    all::{Context, EventHandler, GuildId, VoiceState},
    async_trait,
};
use tracing::error;

pub struct VoiceStateUpdateHandler;

#[async_trait]
impl EventHandler for VoiceStateUpdateHandler {
    // Disconnect from voice channel when everyone leaves
    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        let Some(guild_id) = new_state.guild_id else {
            error!("Received voice state update without guild");
            return;
        };

        let Some(old_state) = old_state else {
            return;
        };

        let Some(channel_id) = old_state.channel_id else {
            error!("Received voice state update without channel");
            return;
        };

        let guild_channels = guild_id.channels(&ctx.http).await.unwrap();

        let guild_channel = guild_channels.get(&channel_id).unwrap();

        let channel_members = guild_channel.members(&ctx.cache).unwrap();

        if channel_members
            .iter()
            .all(|m| m.user.id == ctx.cache.current_user().id)
        {
            leave_channel(&ctx, guild_id).await;
        }
    }
}

async fn leave_channel(ctx: &Context, guild_id: GuildId) {
    let manager = songbird::get(ctx)
        .await
        .expect("Expected songbird voice client to be placed in at initialization");

    // Throws error: `NoCall` for some reason, but successfully disconnects from the call
    let _ = manager.remove(guild_id).await;

    info!("Left voice channel due to member inactivity");
}
