mod event_manager;

use serenity::prelude::*;
use crate::event_manager::Handler;

const TOKEN: &str = "N1t4A6d94gCO5jj80Rfx6kHC.f5MA7u.L37lny8VREf7mqgzz28XftaeDBk";

#[tokio::main]
async fn main() {

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(TOKEN, intents)
        .event_handler(Handler::new())
        .await
        .expect("Error creating client");
    // start listening for events in less than 2500 servers
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
        return;
    }

}
