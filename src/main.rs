use std::error::Error;

use teloxide::{prelude::*, utils::command::{BotCommands, self}, dptree::deps};
use anime_notificator::*;



#[tokio::main]
async fn main() {
   
    //update_serialized_ongoings().await;
    
    let server = Server::default();

    let bot = Bot::new("5548823557:AAGoD89-SXIoPQf4JP1JrEDFNoqkgi9lRgI").auto_send();

    let handler = dptree::entry()
    .branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(answer),
    );

    Dispatcher::builder(bot, handler)
    .dependencies(deps![server])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn answer(
    message : Message,
    bot : AutoSend<Bot>,
    command : Command,
    server : Server
) -> Result<(), Box<dyn Error + Send + Sync>> {
    
    match command {
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Start => {
            bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::FindOngoing(name) => {
            let mut finder = server.anime_ongoings.lock().await;
            let mut res = finder.find_top(&name, 1);
            res.reverse();

            for r in &res {
                bot.send_message(message.chat.id, 
                format!("Аниме: {}\n{}\nОписание: {}", 
                r.name,
                r.url,
                r.desc)).await?;
            }

            if &res.len() == &0 {
                bot.send_message(message.chat.id, 
                format!("Онгоинг \"{}\" не найден:(", name)).await?;
            }
        }
    }

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "вывести это сообщение.")]
    Help,
    #[command(description = "вывести это сообщение.")]
    Start,
    #[command(description = "найти онгоинг.")]
    FindOngoing(String),
}