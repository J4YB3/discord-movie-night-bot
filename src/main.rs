extern crate discord_data;
use discord::{self, Discord as Discord, model as Model};
use std::collections::HashMap;

mod commands;
mod behaviour;

fn main() {
    let mut watch_list: HashMap<String, behaviour::WatchListEntry> = HashMap::new();

    let bot = Discord::from_bot_token(discord_data::TOKEN).expect("Bot creation from token failed");

    let (mut connection, _ready_event) = bot.connect().expect("Establishing connecting to server failed");

    loop {
        match connection.recv_event() {
            Ok(Model::Event::MessageCreate(message)) => {
                // Handle the quit command first, since it needs to be within main
                if message.content == commands::construct(commands::QUIT) {
                    let _ = bot.send_message(
                        message.channel_id,
                        "Quitting. Bye bye.",
                        "",
                        false,
                    );
                    break;
                }
                // Handle all other messages that start with the prefix
                else if message.content.starts_with("!") {
                    call_behaviour(&bot, &message, &mut watch_list);
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
fn call_behaviour(bot: &Discord, message: &Model::Message, watch_list: &mut HashMap<String, behaviour::WatchListEntry>) {
    let mut command: &str = "";
    let mut parameters: &str = "";
    
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
            behaviour::add_movie(bot, message, parameters, watch_list);
        },
        _ => {
            let _ = bot.send_message(
                message.channel_id,
                format!("Unknown command '{}'", command).as_str(),
                "",
                false
            );
        }
    }
}