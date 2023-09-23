mod event_manager;

use event_manager::Handler;
use serenity::prelude::*;

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

    // Start listening for events in less than 2500 servers.
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
