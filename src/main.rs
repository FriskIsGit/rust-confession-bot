mod event_manager;

use event_manager::{ConfessionCount, Handler};
use serenity::prelude::*;
use std::sync::atomic::AtomicUsize;

//    (/.,.                                         . . .
//     *##.,                                     //..*&&
//     ,(#%(.,.                                .**%&&%/,
//      .(##%&(.***(///..../* /*///,   .**** *%&&&&*.** 
//       ,(##%&&&%&&&&&&&&&@&&&&&&&%#/#%&&&&&%%%%(.*    
//       (%&&&&%%%%&&&%%%%%%%%%%%%%%%%%%%###(((*.       
//      #&&&&%%###((((######((((//**//(###%#(/.,        
//    .#%&%%(*..,//(((####(((//*,,,. .,..,(%%(./        
//    %&&%%#*,...,,(%%%%%%###(/**,,..,,*/(((##(              cowabunga.
//    %%%%%%%%%%%%%%%%%%%%%###########((((((((#(.*      
//    ############(##%%%###((###%%%######((//((((**     
//    #(////((((((//////***(((((##((((((((/////(((/,..  
//    /(/***/((((((///*****/((((((///////,,**///(////...
//     /(((#(*,,///*,*//////((((#%##%%/**/(((((((((///(.
//     ,##(((#(,###%(/((/(%%%#%&&&&%///((((((((((((////(
//       (##(((/,(%&&%%&&%&&&&&@&%///((((((((((((///////
//         /##(/*,,(##&&&%&&&((*///((((((((((/////////((
//         /,((((/**,,,,...,**///((((((//////////////(((
//          ,.(###((//****//((((((((////////////////((((

#[tokio::main]
async fn main() {
    let token = std::fs::read_to_string("token.txt").expect("Failed to read token file"); 

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::new())
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        let counter = AtomicUsize::new(1);
        data.insert::<ConfessionCount>(counter);
    }

    // Start listening for events in less than 2500 servers.
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
