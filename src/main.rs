extern crate discord_data;
use discord::{self, Discord as Discord, model as Model};
use std::collections::HashMap;

mod commands;
mod behaviour;

pub struct BotData {
    bot: Discord,
    message: Option<Model::Message>,
    watch_list: HashMap<String, behaviour::WatchListEntry>,
}

fn main() {
    let mut watch_list: HashMap<String, behaviour::WatchListEntry> = HashMap::new();

    let bot = Discord::from_bot_token(discord_data::TOKEN).expect("Bot creation from token failed");

    let (mut connection, _ready_event) = bot.connect().expect("Establishing connecting to server failed");

    let mut bot_data = BotData {
        bot: bot,
        message: None,
        watch_list: watch_list,
    };

    loop {
        match connection.recv_event() {
            Ok(Model::Event::MessageCreate(message)) => {
                // Handle the quit command first, since it needs to be within main
                if message.content == commands::construct(commands::QUIT) {
                    let _ = bot_data.bot.send_message(
                        message.channel_id,
                        "Quitting. Bye bye.",
                        "",
                        false,
                    );
                    break;
                }
                // Handle all other messages that start with the prefix
                else if message.content.starts_with("!") {
                    bot_data.message = Some(message);
                    call_behaviour(&mut bot_data);
                }
            },
            Ok(_) => (),
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}", code, body);
            },
            Err(err) => println!("Received error: {}", err),
        }
    }

    let _ = connection.shutdown();
}

/**
 * Segments the message into the command and the parameters part. Then calls the appropriate
 * behaviour function from behaviour.rs.
 */
fn call_behaviour(bot_data: &mut BotData) {
    let mut command: &str = "";
    let mut parameters: &str = "";

    let bot_data_message = bot_data.message.clone();

    if let Some(message) = bot_data_message {
        // Try to find the first whitespace
        if let Some(first_index) = message.content.find(char::is_whitespace) {
            command = &message.content[1..first_index];
            parameters = &message.content[first_index+1..];
        } else {
            command = &message.content[1..];
        }

        println!("Command was '{}'. Parameters were '{}'", command, parameters);
        match command {
        commands::ADD_MOVIE | commands::ADD_MOVIE_SHORT => {
            behaviour::add_movie(bot_data, parameters);
        },
        _ => {
            let _ = bot_data.bot.send_message(
                message.channel_id,
                format!("Unknown command '{}'", command).as_str(),
                "",
                false
            );
        }
    }
    }
}