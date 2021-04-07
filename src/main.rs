extern crate discord_data;
use discord::{self, Discord as Discord, model as Model, State, model::ServerId};
use std::collections::HashMap;

mod commands;
mod movie_behaviour;
mod general_behaviour;

pub struct BotData {
    bot: Discord,
    message: Option<Model::Message>,
    watch_list: HashMap<u32, movie_behaviour::WatchListEntry>,
    next_movie_id: u32,
    server_id: Model::ServerId,
    server_roles: Vec<Model::Role>,
}

fn main() {
    let mut watch_list: HashMap<u32, movie_behaviour::WatchListEntry> = HashMap::new();

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
                // Handle the quit command first, since it needs to be within main (because of loop break)
                if message.content == commands::construct(commands::QUIT) {
                    let _ = bot_data.bot.send_embed(
                        message.channel_id,
                        "",
                        |embed| embed.description("Quitting. Bye bye.").color(commands::COLOR_BOT)
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

    // If the bot_data parameter received a message continue with processing all possible commands
    if let Some(message) = bot_data_message {
        // Try to find the first whitespace to separate the command from the parameters
        if let Some(first_index) = message.content.find(char::is_whitespace) {
            command = &message.content[1..first_index];
            parameters = &message.content[first_index+1..];
        } else {
            command = &message.content[1..];
        }

        // Show the user that the bot is processing data by broadcasting typing
        let _ = bot_data.bot.broadcast_typing(message.channel_id);

        println!("Command was '{}'. Parameters were '{}'", command, parameters);
        // Match against all available commands
        match command {
            commands::ADD_MOVIE | commands::ADD_MOVIE_SHORT => {
                if parameters == "" {
                    general_behaviour::show_help_add_movie(bot_data);
                } else {
                    movie_behaviour::add_movie(bot_data, parameters);
                }
            },
            commands::REMOVE_MOVIE | commands::REMOVE_MOVIE_SHORT => {
                let desired_id = parameters.parse();
                match desired_id {
                    Ok(id) => movie_behaviour::remove_movie_by_id(bot_data, id),
                    Err(_) => {
                        if parameters == "" {
                            general_behaviour::show_help_remove_movie(bot_data);
                        } else {
                            movie_behaviour::remove_movie_by_title(bot_data, parameters);
                        }
                    }
                };
            },
            commands::EDIT_MOVIE | commands::EDIT_MOVIE_SHORT => {
                // We need to separate the number from the parameters (first parameter)
                // so find the first whitespace in the parameters and parse the first part into a number
                if let Some(first_index) = parameters.find(char::is_whitespace) {
                    let desired_id = parameters[0..first_index].parse::<u32>();
                    match desired_id {
                        // The first parameter could be parsed, so it was a number, proceed
                        Ok(id) => movie_behaviour::edit_movie_by_id(bot_data, id, &parameters[first_index+1..]),

                        // The first parameter was not a number, so show the user the help message for this command
                        Err(_) => general_behaviour::show_help_edit_movie(bot_data),
                    }
                } else {
                    general_behaviour::show_help_edit_movie(bot_data);
                }
            },
            commands::SHOW_WATCH_LIST | commands::SHOW_WATCH_LIST_SHORT => {
                // The watchlist command can not be used incorrectly, so we don't need to show
                // the help for this command in any case
                movie_behaviour::show_watch_list(bot_data);
            },
            commands::HELP | commands::HELP_SHORT => {
                // Match the parameters for any available command
                match parameters {
                    "" => general_behaviour::show_help(bot_data),
                    commands::HELP | commands::HELP_SHORT => general_behaviour::show_help_help(bot_data),
                    commands::QUIT => general_behaviour::show_help_quit(bot_data),
                    commands::ADD_MOVIE | commands::ADD_MOVIE_SHORT => general_behaviour::show_help_add_movie(bot_data),
                    commands::EDIT_MOVIE | commands::EDIT_MOVIE_SHORT => general_behaviour::show_help_edit_movie(bot_data),
                    commands::REMOVE_MOVIE | commands::REMOVE_MOVIE_SHORT => general_behaviour::show_help_remove_movie(bot_data),
                    commands::SHOW_WATCH_LIST | commands::SHOW_WATCH_LIST_SHORT => general_behaviour::show_help_watchlist(bot_data),
                    _ => {
                        let _ = bot_data.bot.send_embed(
                            message.channel_id,
                            "",
                            |embed| embed.description(format!("There is no command `{}` to show help for", parameters).as_str()).color(commands::COLOR_ERROR)
                        );
                    }
                }
            },
            // If the command was not known tell the user
            _ => {
                let _ = bot_data.bot.send_embed(
                    message.channel_id,
                    "",
                    |embed| embed.description(format!("Unknown command `{}`", command).as_str()).color(commands::COLOR_ERROR)
                );
            }
        }
    }
}