use std::{error::Error, ops::Index};

use teloxide::{prelude::*, utils::command::{BotCommands}, dptree::deps, types::InlineKeyboardButton};
use anime_notificator::*;



#[tokio::main]
async fn main() {
   
    // update_serialized_ongoings().await;
    
    let server = Server::default();

    let bot = Bot::new("5548823557:AAGoD89-SXIoPQf4JP1JrEDFNoqkgi9lRgI").auto_send();

    server.save_vec::<BigAnime>("animes", &vec![]).await;

    let bot_running = tokio::spawn(run_bot(
        bot.clone(),
        server.clone()
    ));

    let updating = tokio::spawn(anime_updating(
        bot.clone(),
        server.clone()
    ));

    bot_running.await.unwrap();
    updating.await.unwrap();
}

async fn run_bot(bot : AutoSend<Bot>, server : Server) {
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

async fn anime_updating(bot : AutoSend<Bot>, server : Server) {
    loop {
        println!("Проверка наличия новых серий");

        let users = server.load_vec::<User>("users").await;
        let old_anime = server.load_vec::<BigAnime>("animes").await;
        let new_anime = load_last_anime().await;

        let mut anime_was_updates = false;

        for new_a in &new_anime {
            if !old_anime.contains(new_a) {
                println!("{} получил {} серию", new_a.name, new_a.episode);
                for user in &users {
                    if user.animu_sub.contains(&new_a.name) {
                        println!("Отправка оповещения для {}", user.id);
                        bot.send_message(user.id, 
                            format!("\"{}\" получил {} серию.\n{}", new_a.name, new_a.episode, new_a.url)).await.unwrap();
                    }
                }

                anime_was_updates = true;
            }
        }

        if anime_was_updates {
            server.save_vec("animes", &new_anime).await;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1000 * 60)).await;
    }
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

            InlineKeyboardButton::callback(text, callback_data)

            bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::FindOngoing(name) => {
            let mut finder = server.anime_ongoings.lock().await;
            let mut res = finder.find_top(&name, 1);
            // res.reverse();


            if &res.len() == &0 {
                bot.send_message(message.chat.id, 
                format!("Онгоинг \"{}\" не найден:(", name)).await?;
            } else {
                let r = &res[0];
                bot.send_message(message.chat.id, 
                    format!("Аниме: {}\n{}\nПодписаться на обновления /subsribelast\nОписание: {}", 
                    r.name,
                    r.url,
                    r.desc)).await?;

                let mut user = server.get_user(message.chat.id).await;
                user.last_finded = r.name.clone();
                server.set_user(&user).await;
            }
        }
        Command::SubsribeLast => {
            let mut user = server.get_user(message.chat.id).await;

            if user.last_finded != "" {
                if user.animu_sub.contains(&user.last_finded) == false {
                    user.animu_sub.push(user.last_finded.clone());
                    server.set_user(&user).await;
                    bot.send_message(
                        message.chat.id, 
                        format!("Отлично! Теперь тебе будут приходить обноления о выходе серий онгоинга \"{}\"", user.last_finded.clone())).await?;
                } else {
                    bot.send_message(
                        message.chat.id, 
                        format!("Ты уже подписан на оповещения онгоинга \"{}\"", user.last_finded.clone())).await?;
                }
            } else {
                bot.send_message(
                    message.chat.id, 
                    format!("Для работы этой команды требуется вначале найти онгоинг с помощью /findongoing")).await?;
            }

        }
        Command::UnSubsribeLast => {
            let mut user = server.get_user(message.chat.id).await;

            if user.last_finded != "" {
                if user.animu_sub.contains(&user.last_finded) == false {
                    bot.send_message(
                        message.chat.id, 
                        format!("Ты не подписан на обновления \"{}\"", user.last_finded.clone())).await?;
                } else {
                    let idx = user.animu_sub.iter().position(|r| r == &user.last_finded).unwrap();
                    user.animu_sub.remove(idx);
                    server.set_user(&user).await;
                    bot.send_message(
                        message.chat.id, 
                        format!("Оповещения онгоинга \"{}\" больше приходить не будут", user.last_finded.clone())).await?;
                }
            } else {
                bot.send_message(
                    message.chat.id, 
                    format!("Для работы этой команды требуется вначале найти онгоинг с помощью /findongoing")).await?;
            }
        }
        Command::SubList => {
            let mut user = server.get_user(message.chat.id).await;

            let mut answer = "Ты подписан на:".to_string();

            let mut indexer = 1;

            for title in &user.animu_sub {
                answer = format!("{}\n{}: {}",answer,indexer,title).to_string();
                indexer += 1;
            }

            bot.send_message(
                message.chat.id, answer).await?;
        }
        Command::ClearSubs => {
            let mut user = server.get_user(message.chat.id).await;
            user.animu_sub.clear();
            server.set_user(&user).await;

            bot.send_message(
                message.chat.id, "Больше оповещений приходить от подписок не будет").await?;
        }
        Command::LastAnime => {
            let mut animes = server.load_vec::<BigAnime>("animes").await;
            animes.reverse();

            for anime in animes {
                bot.send_message(message.chat.id, 
                format!("\"{}\" получил {} серию.\n{}", anime.name, anime.episode, anime.url)).await?;
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
    #[command(description = "подписаться на последний найденный онгоинг.")]
    SubsribeLast,
    #[command(description = "отписаться от последнего найденного онгоинга.")]
    UnSubsribeLast,
    #[command(description = "список онгоингов, на которые ты подписан.")]
    SubList,
    #[command(description = "очистить список подписок.")]
    ClearSubs,
    #[command(description = "последние вышедшие аниме.")]
    LastAnime,
    
}