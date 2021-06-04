extern crate external_data;
use discord::{self, Discord as Discord, model as Model, State, model::ServerId};
use std::{collections::HashMap, str::FromStr};
use commands::{Command, ParseCommandError, SimpleCommand};
use tmdb::{themoviedb::*};

mod commands;
mod movie_behaviour;
mod general_behaviour;
mod voting_behaviour;

pub struct BotData {
    bot: Discord,
    message: Option<Model::Message>,
    watch_list: HashMap<u32, movie_behaviour::WatchListEntry>,
    next_movie_id: u32,
    server_id: Model::ServerId,
    server_roles: Vec<Model::Role>,
    custom_prefix: char,
    tmdb: TMDb,
    wait_for_reaction: Vec<general_behaviour::WaitingForReaction>,
    votes: HashMap<u64, voting_behaviour::Vote>, // Keys are the message_ids
}

const COLOR_ERROR: u64 = 0xff0000; // red
const COLOR_SUCCESS: u64 = 0x7ef542; // green
const COLOR_WARNING: u64 = 0xf5d442; // yellow
const COLOR_BOT: u64 = 0xe91e63; // color of the bot role (pink)
const COLOR_INFORMATION: u64 = 0x3b88c3; // blue

fn main() {
    let watch_list: HashMap<u32, movie_behaviour::WatchListEntry> = HashMap::new();
    let votes: HashMap<u64, voting_behaviour::Vote> = HashMap::new();

    let bot = Discord::from_bot_token(external_data::DISCORD_TOKEN).expect("Bot creation from token failed");

    let (mut connection, ready_event) = bot
        .connect()
        .expect("Establishing connecting to server failed");

    let mut state = State::new(ready_event);

    let tmdb = TMDb {
        api_key: external_data::TMDB_API_KEY,
        language: "de",
    };

    let mut bot_data = BotData {
        bot: bot,
        message: None,
        watch_list: watch_list,
        next_movie_id: 0,
        server_id: ServerId(0),
        server_roles: vec![],
        custom_prefix: '!',
        tmdb: tmdb,
        wait_for_reaction: vec![],
        votes: votes,
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

        if state.servers().len() > 0 {
            // Roles could change while the bot is running
            bot_data.server_roles = state.servers()[0].roles.clone();

            if bot_data.server_id == ServerId(0) {
                bot_data.server_id = state.servers()[0].id;
            }
        }

        match event {
            Model::Event::MessageCreate(message) => {
                // If the message is from the bot itself skip this event
                if message.author.id == state.user().id {
                    continue;
                }

                println!("Received message: {:#?}", message.content);

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
                    bot_data.message = Some(message.clone());

                    // Indicate that the bot is processing the query
                    let _ = bot_data.bot.broadcast_typing(message.channel_id);
                    call_behaviour(&mut bot_data);
                }
            },
            Model::Event::ReactionAdd(reaction) => {
                use general_behaviour::WaitingForReaction;
                // If the reaction is from the bot itself skip this event
                if reaction.user_id == state.user().id {
                    continue;
                }

                // Determine if a command is waiting for a reaction
                if bot_data.wait_for_reaction.len() > 0 {
                    for waiting_idx in 0..bot_data.wait_for_reaction.len() {
                        // Get the current element
                        let waiting = bot_data.wait_for_reaction[waiting_idx].clone();
                        match waiting {
                            WaitingForReaction::AddMovie(message_id, new_entry) => {
                                // If the reaction happened to the correct message
                                if reaction.message_id == message_id {
                                    movie_behaviour::add_movie_by_reaction(&mut bot_data, &reaction, &new_entry);

                                    // The correct message was found and has therefore now been reacted to
                                    // Remove the wait_for_reaction element from bot_data and break the loop
                                    bot_data.wait_for_reaction.remove(waiting_idx);
                                    break;
                                }
                            },
                            WaitingForReaction::Vote(message_id) => {
                                // If the reaction happened to the correct message
                                if reaction.message_id == message_id {
                                    voting_behaviour::update_vote(&mut bot_data, &reaction, &message_id.0);

                                    // Vote does not get removed from the wait_for_reaction vector since
                                    // this will only happen once the vote gets closed by the user
                                    // Only break the loop since the correct message was found
                                    break;
                                }
                            },
                        }
                    }
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
        AddMovie(title) => movie_behaviour::search_movie(bot_data, title.as_str(), true),
        RemoveMovieById(id) => movie_behaviour::remove_movie_by_id(bot_data, id),
        RemoveMovieByTitle(title) => {
            movie_behaviour::remove_movie_by_title(bot_data, title.as_str())
        },
        ShowWatchlist(order) => movie_behaviour::show_watch_list(bot_data, order),
        Help(simple_command) => match simple_command {
            SimpleCommand::General => general_behaviour::show_help(bot_data),
            SimpleCommand::Help => general_behaviour::show_help_help(bot_data),
            SimpleCommand::Quit => general_behaviour::show_help_quit(bot_data),
            SimpleCommand::Add => general_behaviour::show_help_add_movie(bot_data),
            SimpleCommand::Remove => general_behaviour::show_help_remove_movie(bot_data),
            SimpleCommand::ShowWatchlist => general_behaviour::show_help_watchlist(bot_data),
            SimpleCommand::Prefix => general_behaviour::show_help_prefix(bot_data),
            SimpleCommand::History => general_behaviour::show_help_history(bot_data),
            SimpleCommand::Status => general_behaviour::show_help_set_status(bot_data),
            SimpleCommand::Unavailable => general_behaviour::show_help_set_status_unavailable(bot_data),
            SimpleCommand::Watched => general_behaviour::show_help_set_status_watched(bot_data),
            SimpleCommand::ShowMovie => general_behaviour::show_help_show_movie(bot_data),
            SimpleCommand::Search => general_behaviour::show_help_search_movie(bot_data),
            SimpleCommand::CreateVote => general_behaviour::show_help_create_vote(bot_data),
            SimpleCommand::SendVote => general_behaviour::show_help_send_vote(bot_data),
            SimpleCommand::CloseVote => general_behaviour::show_help_close_vote(bot_data),
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
        History(order) => movie_behaviour::show_history(bot_data, order),
        SetStatus(id, status) => movie_behaviour::set_status(bot_data, id, status),
        Unavailable(id) => movie_behaviour::set_status(bot_data, id, "Unavailable".to_string()),
        Watched(id, date) => movie_behaviour::set_status_watched(bot_data, id, date),
        ShowMovieById(id) => movie_behaviour::show_movie_by_id(bot_data, id),
        ShowMovieByTitle(title) => movie_behaviour::show_movie_by_title(bot_data, title),
        SearchMovie(title) => movie_behaviour::search_movie(bot_data, title.as_str(), false),
        CreateVote(title, options) => voting_behaviour::create_vote(bot_data, title, options),
        SendVote => voting_behaviour::determine_vote_and_send_details_message(bot_data),
        CloseVote => voting_behaviour::close_vote(bot_data),
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
        NoArgumentsForPrefix => general_behaviour::show_help_prefix(bot_data),
        PrefixIsNotAChar => general_behaviour::show_help_prefix(bot_data),
        WrongArgumentForWatchList => general_behaviour::show_help_watchlist(bot_data),
        WrongArgumentForHistory => general_behaviour::show_help_history(bot_data),
        NotEnoughArgumentsForStatus | WrongArgumentsForStatus => general_behaviour::show_help_set_status(bot_data),
        NoArgumentForUnavailable | WrongArgumentForUnavailable => general_behaviour::show_help_set_status_unavailable(bot_data),
        NotEnoughArgumentsForWatched | WrongArgumentsForWatched => general_behaviour::show_help_set_status_watched(bot_data),
        NoArgumentsForShowMovie => general_behaviour::show_help_show_movie(bot_data),
        NoArgumentsForSearchMovie => general_behaviour::show_help_search_movie(bot_data),
        NoArgumentsForCreateVote => general_behaviour::show_help_create_vote(bot_data),
    }
}