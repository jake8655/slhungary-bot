use poise::CreateReply;
use serenity::all::ChannelId;
use serenity::all::CreateMessage;
use serenity::all::EditMessage;
use serenity::all::Message;
use serenity::all::ReactionType;
use serenity::http::Http;
use tracing::error;

use crate::commands::Context;

pub async fn send_message(
    http: &Http,
    channel_id: ChannelId,
    msg: CreateMessage,
) -> Option<Message> {
    match channel_id.send_message(&http, msg).await {
        Ok(msg) => Some(msg),
        Err(e) => {
            error!("Error sending message: {e:?}");
            None
        }
    }
}

pub async fn delete_message(http: &Http, msg: &Message) {
    if let Err(e) = msg.delete(&http).await {
        error!("Error deleting message: {e:?}");
    }
}

pub async fn edit_message(
    http: &Http,
    channel_id: ChannelId,
    msg_id: u64,
    new_msg: EditMessage,
) -> Option<Message> {
    match channel_id.edit_message(&http, msg_id, new_msg).await {
        Ok(msg) => Some(msg),
        Err(e) => {
            error!("Error editing message: {e:?}");
            None
        }
    }
}

pub async fn send_reply(ctx: &Context<'_>, reply: CreateReply) {
    if let Err(e) = ctx.send(reply).await {
        error!("Error sending reply: {e:?}");
    }
}

pub async fn react_to_message(http: &Http, msg: &Message, reaction: ReactionType) {
    if let Err(e) = msg.react(http, reaction).await {
        error!("Error reacting to message: {e:?}");
    }
}
