extern crate discord_data;
use command::{Command, ParseCommandError, SimpleCommand};
use discord::{self, model as Model, model::ServerId, Discord, State};
use std::{collections::HashMap, str::FromStr};

mod command;
mod general_behaviour;
mod movie_behaviour;

pub struct BotData {
    bot: Discord,
    message: Option<Model::Message>,
    watch_list: HashMap<u32, movie_behaviour::WatchListEntry>,
    next_movie_id: u32,
    server_id: Model::ServerId,
    server_roles: Vec<Model::Role>,
}

const COLOR_ERROR: u64 = 0xff0000; // red
const COLOR_SUCCESS: u64 = 0x7ef542; // green
const COLOR_WARNING: u64 = 0xf5d442; // yellow
const COLOR_BOT: u64 = 0xe91e63; // color of the bot role (pink)
const COLOR_INFORMATION: u64 = 0x3b88c3; // blue

fn main() {
    let watch_list: HashMap<u32, movie_behaviour::WatchListEntry> = HashMap::new();

    let bot = Discord::from_bot_token(discord_data::TOKEN).expect("Bot creation from token failed");

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
                if message.content == Command::Quit.to_string() {
                    let _ = bot_data.bot.send_embed(message.channel_id, "", |embed| {
                        embed.description("Quitting. Bye bye.").color(COLOR_BOT)
                    });
                    break;
                }
                // Handle all other messages that start with the prefix
                else if message.content.starts_with("!") {
                    bot_data.message = Some(message);
                    call_behaviour(&mut bot_data);
                }
            }
            _ => {}
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
        ShowWatchlist => movie_behaviour::show_watch_list(bot_data),
        Help(simple_command) => match simple_command {
            SimpleCommand::General => general_behaviour::show_help(bot_data),
            SimpleCommand::Help => general_behaviour::show_help_help(bot_data),
            SimpleCommand::Quit => general_behaviour::show_help_quit(bot_data),
            SimpleCommand::Add => general_behaviour::show_help_add_movie(bot_data),
            SimpleCommand::Edit => general_behaviour::show_help_edit_movie(bot_data),
            SimpleCommand::Remove => general_behaviour::show_help_remove_movie(bot_data),
            SimpleCommand::Show => general_behaviour::show_help_watchlist(bot_data),
            SimpleCommand::Unknown(parameters) => {
                let _ = bot_data.bot.send_embed(
                    bot_data.message.clone().unwrap().channel_id,
                    "",
                    |embed| {
                        embed
                            .description(
                                format!("There is no command `{}` to show help for", parameters)
                                    .as_str(),
                            )
                            .color(COLOR_ERROR)
                    },
                );
            }
        },
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
                    .description(format!("Unknown command `{}`", message.content).as_str())
                    .color(COLOR_ERROR)
            });
        }
        NoArgumentsForAdd => general_behaviour::show_help_add_movie(bot_data),
        NoArgumentsForRemove => general_behaviour::show_help_remove_movie(bot_data),
        NotEnoughArgumentsForEdit | WrongArgumentForEdit => {
            general_behaviour::show_help_edit_movie(bot_data)
        }
    }
}
