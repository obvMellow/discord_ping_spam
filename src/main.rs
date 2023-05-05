use std::fs::File;
use tokio::task::{self, JoinHandle};

use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    model::prelude::{Channel, ChannelId, GuildId, Message, Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    Client, Error,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Config {
    pub token: String,
    pub guild_id: u64,
    pub ping_id: u64,
}

impl Config {
    pub fn from_reader(reader: impl std::io::Read) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(reader)
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let config = Config::from_reader(File::open("config.json").unwrap()).unwrap();

        let guild = GuildId(config.guild_id);

        let channels = guild.channels(&ctx).await.unwrap();

        let mut handles: Vec<JoinHandle<Result<Message, Error>>> = Vec::new();

        for (channel_id, _) in &channels {
            let channel = ChannelId((*channel_id).into());

            if let Channel::Guild(_) = channel.to_channel(&ctx).await.unwrap() {
                handles.push(spawn(channel, config.clone(), ctx.clone()).await);
            }
        }

        for handle in handles {
            handle.await.unwrap().unwrap();
        }
    }
}

async fn spawn(
    channel_id: ChannelId,
    config: Config,
    ctx: Context,
) -> JoinHandle<Result<Message, Error>> {
    task::spawn(async move {
        loop {
            channel_id
                .say(ctx.http.clone(), format!("<@{}>", config.ping_id))
                .await
                .unwrap();
        }
    })
}

#[tokio::main]
async fn main() {
    let token = Config::from_reader(File::open("config.json").unwrap())
        .unwrap()
        .token;

    let intents = GatewayIntents::all();

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
