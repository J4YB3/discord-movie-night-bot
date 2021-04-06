extern crate discord_data;
use discord::{self, Discord as Discord, model as Model, State, model::ServerId};
use std::collections::HashMap;

mod commands;
mod movie_behaviour;

pub struct BotData {
    bot: Discord,
    message: Option<Model::Message>,
    watch_list: HashMap<String, movie_behaviour::WatchListEntry>,
    next_movie_id: u32,
    server_id: Model::ServerId,
    server_roles: Vec<Model::Role>,
}

fn main() {
    let mut watch_list: HashMap<String, movie_behaviour::WatchListEntry> = HashMap::new();

    let bot = Discord::from_bot_token(discord_data::TOKEN).expect("Bot creation from token failed");

    let (mut connection, ready_event) = bot.connect().expect("Establishing connecting to server failed");

    let mut state = State::new(ready_event);

    let mut bot_data = BotData {
        bot: bot,
        message: None,
        watch_list: watch_list,
        next_movie_id: 0,
        server_id: ServerId(0),
        server_roles: vec![],
    };

    loop {
        let event = match connection.recv_event() {
			Ok(event) => event,
			Err(err) => {
				println!("[Warning] Receive error: {:?}", err);
				if let discord::Error::WebSocket(..) = err {
					// Handle the websocket connection being dropped
					let (connection, ready_event) = bot_data.bot.connect().expect("connect failed");
					state = State::new(ready_event);
					println!("[Ready] Reconnected successfully.");
				}
				if let discord::Error::Closed(..) = err {
					break;
				}
				continue;
			}
		};

        state.update(&event);

        if bot_data.server_id == ServerId(0) {
            bot_data.server_id = state.servers()[0].id;
        }
        // Roles could change while the bot is running
        bot_data.server_roles = state.servers()[0].roles.clone();

        match event {
            Model::Event::MessageCreate(message) => {
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
            _ => {},
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
                movie_behaviour::add_movie(bot_data, parameters);
            },
            commands::REMOVE_MOVIE | commands::REMOVE_MOVIE_SHORT => {
                let desired_id = parameters.parse();
                match desired_id {
                    Ok(id) => movie_behaviour::remove_movie_by_id(bot_data, id),
                    Err(_) => movie_behaviour::remove_movie_by_title(bot_data, parameters)
                };
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