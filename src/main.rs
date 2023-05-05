use std::fs::File;
use tokio::task::{self, JoinHandle};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    model::prelude::{Channel, ChannelId, ChannelType, GuildId, Message, Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    Client, Error,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Config {
    pub token: String,
    pub guild_id: u64,
    pub ping_id: u64,
    pub create_channel: bool,
}

impl Config {
    pub fn from_reader(reader: impl std::io::Read) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(reader)
    }
}

const CHANNEL_NAME_LEN: usize = 50;
const CHANNEL_AMOUNT: usize = 1000;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let config = Config::from_reader(File::open("config.json").unwrap()).unwrap();

        let guild = GuildId(config.guild_id);

        if config.create_channel {
            println!("Creating channels...");

            if CHANNEL_AMOUNT > 500 {
                println!("Cannot create more than 500 channels");
                return;
            }

            if CHANNEL_AMOUNT > 10 {
                let threads = (CHANNEL_AMOUNT - (CHANNEL_AMOUNT % 10)) / 10;
                let remainder = CHANNEL_AMOUNT % 10;

                let mut handles: Vec<JoinHandle<()>> = Vec::new();

                for _ in 0..threads {
                    let ctx = ctx.clone();

                    let handle = task::spawn(async move {
                        for _ in 0..10 {
                            let channel_name: String = rand::thread_rng()
                                .sample_iter(&Alphanumeric)
                                .take(CHANNEL_NAME_LEN)
                                .map(char::from)
                                .collect();

                            guild
                                .create_channel(ctx.http.clone(), |c| {
                                    c.name(channel_name).kind(ChannelType::Text)
                                })
                                .await
                                .unwrap();
                        }
                    });

                    handles.push(handle);
                }

                for _ in 0..remainder {
                    let channel_name: String = rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(CHANNEL_NAME_LEN)
                        .map(char::from)
                        .collect();

                    guild
                        .create_channel(ctx.clone().http.clone(), |c| {
                            c.name(channel_name).kind(ChannelType::Text)
                        })
                        .await
                        .unwrap();
                }
            }

            println!("Done creating channels!");
        }

        let channels = guild.channels(&ctx).await.unwrap();

        let mut handles: Vec<JoinHandle<Result<Message, Error>>> = Vec::new();

        println!("Pinging...");

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
