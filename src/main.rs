extern crate external_data;
use discord::{self, Discord as Discord, model as Model, State, model::ServerId};
use std::{collections::HashMap, str::FromStr};
use commands::{Command, ParseCommandError, SimpleCommand};
use tmdb::{themoviedb::*};
use serde::{Serialize, Deserialize};

mod commands;
mod movie_behaviour;
mod general_behaviour;
mod voting_behaviour;
mod send_message;
mod help_behaviour;
mod history_behaviour;
mod watch_list_behaviour;
mod serde_behaviour;

#[derive(Serialize, Deserialize)]
pub struct BotData {
    #[serde(skip)]
    #[serde(default = "get_default_discord_struct")]
    bot: Discord,

    #[serde(skip)]
    #[serde(default = "get_tmdb_struct")]
    tmdb: TMDb,

    watch_list: HashMap<u32, movie_behaviour::WatchListEntry>,
    wait_for_reaction: Vec<general_behaviour::WaitingForReaction>,
    votes: HashMap<u64, voting_behaviour::Vote>, // Keys are the message_ids
    bot_user: discord::model::User,
    message: Option<Model::Message>,
    next_movie_id: u32,
    server_id: Model::ServerId,
    server_roles: Vec<Model::Role>,
    custom_prefix: char,
    movie_limit_per_user: u32,
    movie_vote_limit: u32,
}

fn get_tmdb_struct() -> TMDb {
    TMDb {
        api_key: external_data::TMDB_API_KEY,
        language: "de",
    }
}

fn get_default_discord_struct() -> Discord {
    Discord::from_bot_token(external_data::DISCORD_TOKEN).expect("Bot creation from token failed")
}

const COLOR_ERROR: u64 = 0xff0000; // red
const COLOR_SUCCESS: u64 = 0x7ef542; // green
const COLOR_WARNING: u64 = 0xf5d442; // yellow
const COLOR_BOT: u64 = 0xe91e63; // color of the bot role (pink)
const COLOR_INFORMATION: u64 = 0x3b88c3; // blue

const MAX_ENTRIES_PER_PAGE: usize = 10;
const VERSION: &str = "0.5.1";

fn main() {
    let bot = get_default_discord_struct();

    let (mut connection, ready_event) = bot
        .connect()
        .expect("Establishing connection to server failed");

    let mut state = State::new(ready_event);

    let tmdb = get_tmdb_struct();

    let state_user = state.user();

    let mut bot_data: BotData;
    match serde_behaviour::read_bot_data() {
        Ok(data) => {
            bot_data = data;
            // Fill the struct with the data that must be created anew on every start
            bot_data.bot = bot;
            bot_data.bot_user = Model::User {
                id: state_user.id,
                name: state_user.username.clone(),
                discriminator: state_user.discriminator,
                avatar: state_user.avatar.clone(),
                bot: state_user.bot,
            };
            bot_data.tmdb = tmdb;
        },
        Err(string) => {
            println!("{}\n", string);
            println!("WARNING: New BotData created, because file was empty or an error occured!");
            println!("Do you want to proceed, and risk losing data? [y/n]");
            let mut answer = String::new();
            let _ = std::io::stdin().read_line(&mut answer).unwrap();
            println!("Answer was: {}", answer);
            if answer.trim() == "y" {
                bot_data = BotData {
                    bot: bot,
                    bot_user: Model::User {
                        id: state_user.id,
                        name: state_user.username.clone(),
                        discriminator: state_user.discriminator,
                        avatar: state_user.avatar.clone(),
                        bot: state_user.bot,
                    },
                    message: None,
                    watch_list: HashMap::new(),
                    next_movie_id: 0,
                    server_id: ServerId(0),
                    server_roles: vec![],
                    custom_prefix: '!',
                    tmdb: tmdb,
                    wait_for_reaction: vec![],
                    votes: HashMap::new(),
                    movie_limit_per_user: 10,
                    movie_vote_limit: 2,
                };
            } else {
                println!("Bot is shutting down now.");
                return;
            }
        },
    };

    let one_hour = std::time::Duration::from_secs(3600);
    let mut last_save = std::time::Instant::now();
    let mut something_changed = false;

    loop {
        // The last save was more than an hour ago
        if something_changed && last_save.elapsed() >= one_hour {
            // So save the bot_data and reset the last_save time
            serde_behaviour::store_bot_data(&bot_data);
            last_save = std::time::Instant::now();
            something_changed = false;
        }

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
                if message.content == String::from(format!("{}{}", bot_data.custom_prefix, crate::commands::QUIT)) {
                    serde_behaviour::store_bot_data(&bot_data);
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
                    something_changed = true;
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
                                    something_changed = true;
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
                                    something_changed = true;
                                    break;
                                }
                            },
                            WaitingForReaction::AddMovieToWatched(message_id, movie) => {
                                if reaction.message_id == message_id {
                                    movie_behaviour::handle_add_movie_to_watched_after_movie_vote(
                                        &mut bot_data, 
                                        &reaction, 
                                        &movie
                                    );

                                    // Vote does not get removed from the wait_for_reaction vector since
                                    // this will only happen once the vote gets closed by the user
                                    // Only break the loop since the correct message was found
                                    something_changed = true;
                                    break;
                                }
                            },
                            WaitingForReaction::WatchListPagination(message_id, sorted_watch_list_enum, curr_page) => {
                                if reaction.message_id == message_id {
                                    movie_behaviour::handle_watch_list_message_pagination_reaction(
                                        &mut bot_data,
                                        message_id, 
                                        sorted_watch_list_enum, 
                                        curr_page, 
                                        &reaction
                                    );
                                    something_changed = true;
                                }
                            },
                            WaitingForReaction::HistoryPagination(message_id, sorted_history_enum, curr_page) => {
                                if reaction.message_id == message_id {
                                    movie_behaviour::handle_watch_list_message_pagination_reaction(
                                        &mut bot_data,
                                        message_id, 
                                        sorted_history_enum, 
                                        curr_page, 
                                        &reaction
                                    );
                                    something_changed = true;
                                }
                            }
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
        ShowWatchlist(order) => watch_list_behaviour::show_watch_list(bot_data, order),
        Help(simple_command) => match simple_command {
            SimpleCommand::General => help_behaviour::show_help(bot_data),
            SimpleCommand::Help => help_behaviour::show_help_help(bot_data),
            SimpleCommand::Quit => help_behaviour::show_help_quit(bot_data),
            SimpleCommand::Add => help_behaviour::show_help_add_movie(bot_data),
            SimpleCommand::Remove => help_behaviour::show_help_remove_movie(bot_data),
            SimpleCommand::ShowWatchlist => help_behaviour::show_help_watchlist(bot_data),
            SimpleCommand::Prefix => help_behaviour::show_help_prefix(bot_data),
            SimpleCommand::History => help_behaviour::show_help_history(bot_data),
            SimpleCommand::Status => help_behaviour::show_help_set_status(bot_data),
            SimpleCommand::Unavailable => help_behaviour::show_help_set_status_unavailable(bot_data),
            SimpleCommand::Watched => help_behaviour::show_help_set_status_watched(bot_data),
            SimpleCommand::ShowMovie => help_behaviour::show_help_show_movie(bot_data),
            SimpleCommand::Search => help_behaviour::show_help_search_movie(bot_data),
            SimpleCommand::CreateVote => help_behaviour::show_help_create_vote(bot_data),
            SimpleCommand::SendVote => help_behaviour::show_help_send_vote(bot_data),
            SimpleCommand::CloseVote => help_behaviour::show_help_close_vote(bot_data),
            SimpleCommand::MovieLimit => help_behaviour::show_help_movie_limit(bot_data),
            SimpleCommand::MovieVoteLimit => help_behaviour::show_help_movie_vote_limit(bot_data),
            SimpleCommand::RandomMovieVote => help_behaviour::show_help_random_movie_vote(bot_data),
            SimpleCommand::CloseMovieVote => help_behaviour::show_help_close_movie_vote(bot_data),
            SimpleCommand::Info => help_behaviour::show_help_info(bot_data),
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
        History(order) => history_behaviour::show_history(bot_data, order),
        SetStatus(id, status) => movie_behaviour::set_status(bot_data, id, status),
        Unavailable(id) => movie_behaviour::set_status(bot_data, id, "Unavailable".to_string()),
        Watched(id, date) => movie_behaviour::set_status_watched(bot_data, id, date),
        ShowMovieById(id) => movie_behaviour::show_movie_by_id(bot_data, id),
        ShowMovieByTitle(title) => movie_behaviour::show_movie_by_title(bot_data, title),
        SearchMovie(title) => movie_behaviour::search_movie(bot_data, title.as_str(), false),
        CreateVote(title, options) => voting_behaviour::create_vote(bot_data, title, options, false),
        SendVote => voting_behaviour::determine_vote_and_send_details_message(bot_data, None),
        SendVoteWithUserId(user_id) => voting_behaviour::determine_vote_and_send_details_message(bot_data, Some(user_id)),
        CloseVote => voting_behaviour::close_vote(bot_data),
        SetMovieLimit(number) => movie_behaviour::set_movie_limit(bot_data, number),
        ShowMovieLimit => movie_behaviour::show_movie_limit(bot_data),
        SetMovieVoteLimit(number) => voting_behaviour::set_movie_vote_limit(bot_data, number),
        ShowMovieVoteLimit => voting_behaviour::show_movie_vote_limit(bot_data),
        RandomMovieVote(optional_limit) => voting_behaviour::create_random_movie_vote(bot_data, optional_limit),
        CloseMovieVote => voting_behaviour::close_random_movie_vote(bot_data),
        Info => send_message::info(bot_data),
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
        NoArgumentsForAdd => help_behaviour::show_help_add_movie(bot_data),
        NoArgumentsForRemove => help_behaviour::show_help_remove_movie(bot_data),
        NoArgumentsForPrefix => help_behaviour::show_help_prefix(bot_data),
        PrefixIsNotAChar => help_behaviour::show_help_prefix(bot_data),
        WrongArgumentForWatchList => help_behaviour::show_help_watchlist(bot_data),
        WrongArgumentForHistory => help_behaviour::show_help_history(bot_data),
        NotEnoughArgumentsForStatus | WrongArgumentsForStatus => help_behaviour::show_help_set_status(bot_data),
        NoArgumentForUnavailable | WrongArgumentForUnavailable => help_behaviour::show_help_set_status_unavailable(bot_data),
        NotEnoughArgumentsForWatched | WrongArgumentsForWatched => help_behaviour::show_help_set_status_watched(bot_data),
        NoArgumentsForShowMovie => help_behaviour::show_help_show_movie(bot_data),
        NoArgumentsForSearchMovie => help_behaviour::show_help_search_movie(bot_data),
        NoArgumentsForCreateVote => help_behaviour::show_help_create_vote(bot_data),
        WrongArgumentsForMovieLimit => help_behaviour::show_help_movie_limit(bot_data),
        WrongArgumentsForMovieVoteLimit => help_behaviour::show_help_movie_vote_limit(bot_data),
        WrongArgumentsForSendVoteWithUserId => help_behaviour::show_help_send_vote(bot_data),
        WrongArgumentForRandomMovieVote => help_behaviour::show_help_random_movie_vote(bot_data),
    }
}