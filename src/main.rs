extern crate external_data;
use discord::{self, Discord as Discord, model as Model, State, model::ServerId};
use std::{collections::HashMap, str::FromStr};
use commands::{Command, ParseCommandError, SimpleCommand};

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
    custom_prefix: char,
}

const COLOR_ERROR: u64 = 0xff0000; // red
const COLOR_SUCCESS: u64 = 0x7ef542; // green
const COLOR_WARNING: u64 = 0xf5d442; // yellow
const COLOR_BOT: u64 = 0xe91e63; // color of the bot role (pink)
const COLOR_INFORMATION: u64 = 0x3b88c3; // blue

fn main() {
    let watch_list: HashMap<u32, movie_behaviour::WatchListEntry> = HashMap::new();

    let bot = Discord::from_bot_token(external_data::DISCORD_TOKEN).expect("Bot creation from token failed");

    let (mut connection, ready_event) = bot
        .connect()
        .expect("Establishing connecting to server failed");

    let mut state = State::new(ready_event);

    let mut bot_data = BotData {
        bot: bot,
        message: None,
        watch_list: watch_list,
        next_movie_id: 0,
        server_id: ServerId(0),
        server_roles: vec![],
        custom_prefix: '!',
    };

    loop {
        let event = match connection.recv_event() {
			Ok(event) => event,
			Err(err) => {
				println!("[Warning] Receive error: {:?}", err);
				if let discord::Error::WebSocket(..) = err {
					// Handle the websocket connection being dropped
					let (_connection, ready_event) = 
                        bot_data.bot.connect().expect("connect failed");
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
                if message.content == Command::Quit.to_string(&bot_data) {
                    let _ = bot_data.bot.send_embed(
                        message.channel_id,
                        "",
                        |embed| embed.description("Ich beende mich dann mal. Tschüss. :wave:").color(COLOR_BOT)
                    );
                    break;
                }
                // Handle all other messages that start with the prefix
                else if message.content.starts_with(bot_data.custom_prefix) {
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
#[allow(unused_assignments)]
fn call_behaviour(bot_data: &mut BotData) {
    
    if bot_data.message.is_none() {
        return;
    }

    let command_str = bot_data.message.as_ref().unwrap().content.clone();
    let command_result = Command::from_str(command_str.as_str());
    match command_result {
        Ok(command) => handle_command(bot_data, command),
        Err(error) => handle_error(bot_data, error),
    }
}

fn handle_command(bot_data: &mut BotData, command: Command) {
    use Command::*;
    match command {
        AddMovie(title) => movie_behaviour::add_movie(bot_data, title.as_str()),
        RemoveMovieById(id) => movie_behaviour::remove_movie_by_id(bot_data, id),
        RemoveMovieByTitle(title) => {
            movie_behaviour::remove_movie_by_title(bot_data, title.as_str())
        }
        EditMovie(id, new_title) => {
            movie_behaviour::edit_movie_by_id(bot_data, id, new_title.as_str())
        }
        ShowWatchlist(order) => movie_behaviour::show_watch_list(bot_data, order),
        Help(simple_command) => match simple_command {
            SimpleCommand::General => general_behaviour::show_help(bot_data),
            SimpleCommand::Help => general_behaviour::show_help_help(bot_data),
            SimpleCommand::Quit => general_behaviour::show_help_quit(bot_data),
            SimpleCommand::Add => general_behaviour::show_help_add_movie(bot_data),
            SimpleCommand::Edit => general_behaviour::show_help_edit_movie(bot_data),
            SimpleCommand::Remove => general_behaviour::show_help_remove_movie(bot_data),
            SimpleCommand::Show => general_behaviour::show_help_watchlist(bot_data),
            SimpleCommand::Prefix => general_behaviour::show_help_prefix(bot_data),
            SimpleCommand::Unknown(parameters) => {
                let _ = bot_data.bot.send_embed(
                    bot_data.message.clone().unwrap().channel_id,
                    "",
                    |embed| {
                        embed
                            .description(
                                format!("Das Kommando `{}` existiert nicht. Deshalb kann ich dir leider keine Hilfe anzeigen.", parameters)
                                    .as_str(),
                            )
                            .color(COLOR_ERROR)
                    },
                );
            }
        },
        Prefix(new_prefix) => general_behaviour::set_new_prefix(bot_data, new_prefix),
        Quit => todo!("What needs to happen when the Quit command is received?"),
    }
}

fn handle_error(bot_data: &BotData, error: ParseCommandError) {
    use ParseCommandError::*;
    match error {
        NoCommand => {}
        UnknownCommand => {
            let message = bot_data.message.clone().unwrap();
            let _ = bot_data.bot.send_embed(message.channel_id, "", |embed| {
                embed
                    .description(format!("Unbekanntes Kommando `{}`. Vielleicht vertippt? :see_no_evil:", message.content).as_str())
                    .color(COLOR_ERROR)
            });
        }
        NoArgumentsForAdd => general_behaviour::show_help_add_movie(bot_data),
        NoArgumentsForRemove => general_behaviour::show_help_remove_movie(bot_data),
        NotEnoughArgumentsForEdit | WrongArgumentsForEdit => {
            general_behaviour::show_help_edit_movie(bot_data)
        },
        NoArgumentsForPrefix => general_behaviour::show_help_prefix(bot_data),
        PrefixIsNotAChar => general_behaviour::show_help_prefix(bot_data),
        WrongArgumentForWatchList => general_behaviour::show_help_watchlist(bot_data),
    }
}