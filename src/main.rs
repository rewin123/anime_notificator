use teloxide::{prelude::*, utils::command::{BotCommands, self}};
use anime_notificator::*;



#[tokio::main]
async fn main() {
   
    //update_serialized_ongoings().await;
    
    let server = Server::default();

    let bot = Bot::new("5548823557:AAGoD89-SXIoPQf4JP1JrEDFNoqkgi9lRgI").auto_send();

    // let handler = dptree::entry()
    // .branch(
    //     Update::filter_message()
    //         .filter_command::<Command>()
    //         .endpoint(commands),
    // );
}

fn answer(
    message : Message,
    bot : AutoSend<Bot>,
    command : Command,
    server : Server
) {
    
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}