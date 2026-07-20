use commands::{Command, ParseCommandError, SimpleCommand};
use discord::{self, Discord, State, model as Model, model::ServerId};
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    error::Error,
    io::Write,
    panic,
    str::FromStr,
    thread,
    time::{Duration, Instant},
};
use tmdb::themoviedb::*;
use tracing_subscriber::EnvFilter;

mod commands;
mod config;
mod general_behaviour;
mod help_behaviour;
mod history_behaviour;
mod movie_behaviour;
mod reconnect;
mod send_message;
mod serde_behaviour;
mod voting_behaviour;
mod watch_list_behaviour;

#[derive(Serialize)]
pub struct BotData {
    #[serde(skip)]
    bot: Discord,

    #[serde(skip)]
    #[serde(default = "get_tmdb_struct")]
    tmdb: TMDb,

    #[serde(default)]
    watch_list: HashMap<u32, movie_behaviour::WatchListEntry>, // Keys are the internal movie ids

    #[serde(skip)]
    #[serde(default)]
    wait_for_reaction: Vec<general_behaviour::WaitingForReaction>,

    #[serde(default)]
    votes: HashMap<u64, voting_behaviour::Vote>, // Keys are the message_ids

    #[serde(default = "get_default_bot_user")]
    bot_user: discord::model::User,

    #[serde(default)]
    message: Option<Model::Message>,

    #[serde(default)]
    server_roles: Vec<Model::Role>,

    #[serde(default = "get_default_server_id")]
    server_id: Model::ServerId,

    #[serde(skip)]
    #[serde(default)]
    adding_movie: Option<std::time::Instant>,

    custom_prefix: char,
    movie_limit_per_user: u32,
    movie_vote_limit: u32,
    next_movie_id: u32,
}

fn get_tmdb_struct() -> TMDb {
    TMDb {
        api_key: config::get().tmdb_api_key(),
        language: "de",
    }
}

#[derive(Deserialize)]
struct PersistedBotData {
    #[serde(default)]
    watch_list: HashMap<u32, movie_behaviour::WatchListEntry>,

    #[serde(default)]
    wait_for_reaction: Vec<general_behaviour::WaitingForReaction>,

    #[serde(default)]
    votes: HashMap<u64, voting_behaviour::Vote>,

    #[serde(default = "get_default_bot_user")]
    bot_user: discord::model::User,

    #[serde(default)]
    message: Option<Model::Message>,

    #[serde(default)]
    server_roles: Vec<Model::Role>,

    #[serde(default = "get_default_server_id")]
    server_id: Model::ServerId,

    custom_prefix: char,
    movie_limit_per_user: u32,
    movie_vote_limit: u32,
    next_movie_id: u32,
}

impl<'de> Deserialize<'de> for BotData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_bot_data(deserializer, create_discord_client)
    }
}

fn create_discord_client() -> discord::Result<Discord> {
    Discord::from_bot_token(config::get().discord_token())
}

fn deserialize_bot_data<'de, D, E>(
    deserializer: D,
    create_client: impl FnOnce() -> Result<Discord, E>,
) -> Result<BotData, D::Error>
where
    D: serde::Deserializer<'de>,
    E: std::fmt::Display,
{
    let persisted = PersistedBotData::deserialize(deserializer)?;
    let bot = create_client().map_err(serde::de::Error::custom)?;

    Ok(BotData {
        bot,
        tmdb: get_tmdb_struct(),
        watch_list: persisted.watch_list,
        wait_for_reaction: persisted.wait_for_reaction,
        votes: persisted.votes,
        bot_user: persisted.bot_user,
        message: persisted.message,
        server_roles: persisted.server_roles,
        server_id: persisted.server_id,
        adding_movie: None,
        custom_prefix: persisted.custom_prefix,
        movie_limit_per_user: persisted.movie_limit_per_user,
        movie_vote_limit: persisted.movie_vote_limit,
        next_movie_id: persisted.next_movie_id,
    })
}

fn get_default_bot_user() -> discord::model::User {
    discord::model::User {
        id: discord::model::UserId(827634208204783627),
        name: String::from("Movie Night Bot"),
        discriminator: 7301,
        avatar: Some(String::from("7c8c90ae23711af29b91880b44fdfd4a")),
        bot: true,
    }
}

fn get_default_server_id() -> discord::model::ServerId {
    discord::model::ServerId(0)
}

const COLOR_ERROR: u64 = 0xff0000; // red
const COLOR_SUCCESS: u64 = 0x7ef542; // green
const COLOR_WARNING: u64 = 0xf5d442; // yellow
const COLOR_BOT: u64 = 0xe91e63; // color of the bot role (pink)
const COLOR_INFORMATION: u64 = 0x3b88c3; // blue

const MAX_ENTRIES_PER_PAGE: usize = 10;
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn Error>> {
    config::initialize()?;
    initialize_observability()?;
    run()
}

fn initialize_observability() -> Result<(), Box<dyn Error>> {
    let filter = match config::get().log_filter() {
        Some(filter) => EnvFilter::try_new(filter)?,
        None => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(std::io::Error::other)?;

    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let location = panic_info.location();
        tracing::error!(
            event = "panic",
            payload = panic_payload_classification(panic_info.payload()),
            location_file = location.map_or("unknown", std::panic::Location::file),
            location_line = location.map_or(0, std::panic::Location::line),
            "application panicked"
        );
        default_hook(panic_info);
    }));

    Ok(())
}

fn panic_payload_classification(payload: &(dyn Any + Send)) -> &'static str {
    if payload.is::<String>() {
        "string"
    } else if payload.is::<&str>() {
        "string_slice"
    } else {
        "non_string"
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    tracing::info!(event = "application_starting", "application starting");
    let bot = create_discord_client()?;

    let (mut connection, ready_event) = bot.connect()?;
    let mut state = State::new(ready_event);
    tracing::info!(
        event = "discord_connected",
        phase = "initial",
        bot_user_id = state.user().id.0,
        "Discord connected"
    );

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
        }
        Err(string) => {
            tracing::warn!(
                event = "bot_data_load_failed",
                "creating new bot data requires confirmation"
            );
            eprintln!("{string}\n");
            eprint!(
                "WARNING: New BotData created, because file was empty or an error occured!\nDo you want to proceed, and risk losing data? [y/n]\n"
            );
            std::io::stderr().flush()?;
            let mut answer = String::new();
            let bytes_read = std::io::stdin().read_line(&mut answer)?;
            tracing::debug!(event = "bot_data_confirmation_received", bytes_read);
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
                    custom_prefix: '.',
                    tmdb: tmdb,
                    wait_for_reaction: vec![],
                    votes: HashMap::new(),
                    movie_limit_per_user: 10,
                    movie_vote_limit: 2,
                    adding_movie: None,
                };
                tracing::info!(event = "bot_started_with_new_data");
            } else {
                tracing::info!(event = "startup_cancelled");
                return Ok(());
            }
        }
    };

    let thirty_seconds = Duration::from_secs(30);
    let one_hour = Duration::from_secs(3600);
    let mut last_save = Instant::now();
    let mut something_changed = false;

    loop {
        // The last save was more than an hour ago
        if last_save.elapsed() >= one_hour {
            last_save = Instant::now();

            if something_changed {
                if serde_behaviour::store_bot_data_silently(&bot_data).is_ok() {
                    something_changed = false;
                } else {
                    tracing::warn!(
                        event = "hourly_bot_data_persistence_failed",
                        "hourly bot data persistence failed"
                    );
                }
            }
        }

        // See if an add_movie command is waiting too long
        if let Some(start_time) = bot_data.adding_movie {
            if start_time.elapsed() >= thirty_seconds {
                // If so, remove the reactions from the message and remove the
                // add_movie enum from waiting_for_reaction
                let result =
                    bot_data
                        .wait_for_reaction
                        .iter()
                        .enumerate()
                        .find_map(|(idx, entry)| {
                            if let general_behaviour::WaitingForReaction::AddMovie(message, _) =
                                entry
                            {
                                Some((idx, message))
                            } else {
                                None
                            }
                        });

                // If an entry that matches was found
                if let Some((index, message)) = result {
                    // Remove the reactions
                    general_behaviour::remove_reactions_on_message(
                        &bot_data,
                        message,
                        vec!["✅", "❎"],
                    );
                    bot_data.wait_for_reaction.remove(index);

                    send_message::adding_movie_timed_out_information(&bot_data);
                }
            }
        }

        let event = match connection.recv_event() {
            Ok(event) => event,
            Err(err) => {
                tracing::warn!(
                    event = "discord_event_receive_failed",
                    "failed to receive Discord event"
                );
                if matches!(
                    err,
                    discord::Error::WebSocket(..) | discord::Error::Closed(..)
                ) {
                    match reconnect::reconnect_with_backoff(
                        || bot_data.bot.connect(),
                        thread::sleep,
                        reconnect::RECONNECT_DELAYS,
                    ) {
                        Ok((new_connection, ready_event)) => {
                            connection = new_connection;
                            state = State::new(ready_event);
                            tracing::info!(
                                event = "discord_reconnected",
                                "Discord connection restored"
                            );
                        }
                        Err(reconnect_error) => return Err(Box::new(reconnect_error)),
                    }
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

                tracing::debug!(
                    event = "discord_message_received",
                    "received Discord message event"
                );

                // Handle the quit command first, since it needs to be within main (because of loop break)
                if message.content
                    == String::from(format!(
                        "{}{}",
                        bot_data.custom_prefix,
                        crate::commands::QUIT
                    ))
                {
                    bot_data.message = Some(message.clone());
                    if serde_behaviour::store_bot_data(&bot_data).is_err() {
                        tracing::warn!(
                            event = "quit_bot_data_persistence_failed",
                            "failed to persist bot data before quitting"
                        );
                    }

                    general_behaviour::remove_all_reactions_on_all_waiting_for_reaction_messages(
                        &bot_data,
                    );

                    if bot_data
                        .bot
                        .send_embed(message.channel_id, "", |embed| {
                            embed
                                .description("Ich beende mich dann mal. Tschüss. :wave:")
                                .color(COLOR_BOT)
                        })
                        .is_err()
                    {
                        tracing::warn!(
                            event = "quit_confirmation_send_failed",
                            "failed to send quit confirmation"
                        );
                    }
                    break;
                }
                // Handle all other messages that start with the prefix
                else if message.content.starts_with(bot_data.custom_prefix) {
                    bot_data.message = Some(message.clone());

                    // Indicate that the bot is processing the query
                    if bot_data.bot.broadcast_typing(message.channel_id).is_err() {
                        tracing::debug!(
                            event = "typing_indicator_failed",
                            "failed to broadcast typing indicator"
                        );
                    }
                    call_behaviour(&mut bot_data);
                    something_changed = true;
                }
            }
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
                            WaitingForReaction::AddMovie(message, new_entry) => {
                                // If the reaction happened to the correct message
                                if reaction.message_id == message.id {
                                    movie_behaviour::add_movie_by_reaction(
                                        &mut bot_data,
                                        &reaction,
                                        &new_entry,
                                    );

                                    // The correct message was found and has therefore now been reacted to
                                    // Remove the wait_for_reaction element from bot_data and break the loop
                                    bot_data.wait_for_reaction.remove(waiting_idx);
                                    something_changed = true;
                                    break;
                                }
                            }
                            WaitingForReaction::Vote(message) => {
                                // If the reaction happened to the correct message
                                if reaction.message_id == message.id {
                                    voting_behaviour::update_vote(
                                        &mut bot_data,
                                        &reaction,
                                        &message.id.0,
                                    );

                                    // Vote does not get removed from the wait_for_reaction vector since
                                    // this will only happen once the vote gets closed by the user
                                    // Only break the loop since the correct message was found
                                    something_changed = true;
                                    break;
                                }
                            }
                            WaitingForReaction::AddMovieToWatched(message, movie) => {
                                if reaction.message_id == message.id {
                                    movie_behaviour::handle_add_movie_to_watched_after_movie_vote(
                                        &mut bot_data,
                                        &reaction,
                                        &movie,
                                    );

                                    // Vote does not get removed from the wait_for_reaction vector since
                                    // this will only happen once the vote gets closed by the user
                                    // Only break the loop since the correct message was found
                                    something_changed = true;
                                    break;
                                }
                            }
                            WaitingForReaction::WatchListPagination(
                                message,
                                sorted_watch_list_enum,
                                curr_page,
                            ) => {
                                if reaction.message_id == message.id {
                                    movie_behaviour::handle_watch_list_message_pagination_reaction(
                                        &mut bot_data,
                                        message,
                                        sorted_watch_list_enum,
                                        curr_page,
                                        &reaction,
                                    );
                                    something_changed = true;
                                }
                            }
                            WaitingForReaction::HistoryPagination(
                                message,
                                sorted_history_enum,
                                curr_page,
                            ) => {
                                if reaction.message_id == message.id {
                                    movie_behaviour::handle_watch_list_message_pagination_reaction(
                                        &mut bot_data,
                                        message,
                                        sorted_history_enum,
                                        curr_page,
                                        &reaction,
                                    );
                                    something_changed = true;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    tracing::info!(event = "application_stopping", "application stopping");
    if connection.shutdown().is_err() {
        tracing::warn!(
            event = "discord_shutdown_failed",
            "failed to close Discord connection"
        );
    }

    Ok(())
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

    let Some(message) = bot_data.message.as_ref() else {
        tracing::warn!(
            event = "command_failed",
            command = "unknown",
            "command dispatch lacked message context"
        );
        return;
    };
    let command_result = Command::from_str(&message.content);
    match command_result {
        Ok(command) => {
            let command_name = command.name();
            tracing::info!(
                event = "command_started",
                command = command_name,
                "command started"
            );
            handle_command(bot_data, command);
            tracing::info!(
                event = "command_succeeded",
                command = command_name,
                "command completed"
            );
        }
        Err(error) => {
            tracing::info!(
                event = "command_rejected",
                command = "unknown",
                "command rejected"
            );
            handle_error(bot_data, error);
        }
    }
}

fn handle_command(bot_data: &mut BotData, command: Command) {
    use Command::*;
    match command {
        AddMovie(title) => movie_behaviour::search_movie(bot_data, title.as_str(), true),
        RemoveMovieById(id) => movie_behaviour::remove_movie_by_id(bot_data, id),
        RemoveMovieByTitle(title) => {
            movie_behaviour::remove_movie_by_title(bot_data, title.as_str())
        }
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
            SimpleCommand::Status => help_behaviour::show_help_status(bot_data),
            SimpleCommand::Unavailable => {
                help_behaviour::show_help_set_status_unavailable(bot_data)
            }
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
            SimpleCommand::Save => help_behaviour::show_help_save(bot_data),
            SimpleCommand::Count => help_behaviour::show_help_count_movies(bot_data),
            SimpleCommand::Unknown(parameters) => {
                if bot_data
                    .bot
                    .send_embed(
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
                    )
                    .is_err()
                {
                    tracing::warn!(event = "help_response_send_failed", "failed to send help response");
                }
            }
        },
        Prefix(new_prefix) => general_behaviour::set_new_prefix(bot_data, new_prefix),
        History(order) => history_behaviour::show_history(bot_data, order, true),
        SetStatus(id, status) => movie_behaviour::set_status(bot_data, id, status),
        Unavailable(id) => movie_behaviour::set_status(bot_data, id, "Unavailable".to_string()),
        Watched(id, date) => movie_behaviour::set_status_watched(bot_data, id, date),
        ShowMovieById(id) => movie_behaviour::show_movie_by_id(bot_data, id),
        ShowMovieByTitle(title) => movie_behaviour::show_movie_by_title(bot_data, title),
        SearchMovie(title) => movie_behaviour::search_movie(bot_data, title.as_str(), false),
        CreateVote(title, options) => {
            voting_behaviour::create_vote(bot_data, title, options, false)
        }
        SendVote => voting_behaviour::determine_vote_and_send_details_message(bot_data, None),
        SendVoteWithUserId(user_id) => {
            voting_behaviour::determine_vote_and_send_details_message(bot_data, Some(user_id))
        }
        CloseVote => voting_behaviour::close_vote(bot_data),
        SetMovieLimit(number) => movie_behaviour::set_movie_limit(bot_data, number),
        ShowMovieLimit => movie_behaviour::show_movie_limit(bot_data),
        SetMovieVoteLimit(number) => voting_behaviour::set_movie_vote_limit(bot_data, number),
        ShowMovieVoteLimit => voting_behaviour::show_movie_vote_limit(bot_data),
        RandomMovieVote(optional_limit) => {
            voting_behaviour::create_random_movie_vote(bot_data, optional_limit)
        }
        CloseMovieVote => voting_behaviour::close_random_movie_vote(bot_data),
        Info => send_message::info(bot_data),
        Save => {
            if serde_behaviour::store_bot_data(bot_data).is_err() {
                tracing::warn!(
                    event = "command_bot_data_persistence_failed",
                    "failed to persist bot data for save command"
                );
            }
        }
        Count => movie_behaviour::count_movies(bot_data),
        Quit => tracing::warn!(
            event = "quit_command_unexpected_dispatch",
            "quit command should be handled by the main event loop"
        ),
    }
}

fn handle_error(bot_data: &BotData, error: ParseCommandError) {
    use ParseCommandError::*;
    match error {
        NoCommand => {}
        UnknownCommand => {
            let message = bot_data.message.clone().unwrap();
            if bot_data
                .bot
                .send_embed(message.channel_id, "", |embed| {
                    embed
                        .description(
                            format!(
                                "Unbekanntes Kommando `{}`. Vielleicht vertippt? :see_no_evil:",
                                message.content
                            )
                            .as_str(),
                        )
                        .color(COLOR_ERROR)
                })
                .is_err()
            {
                tracing::warn!(
                    event = "unknown_command_response_send_failed",
                    "failed to send unknown command response"
                );
            }
        }
        NoArgumentsForAdd => help_behaviour::show_help_add_movie(bot_data),
        NoArgumentsForRemove => help_behaviour::show_help_remove_movie(bot_data),
        NoArgumentsForPrefix => help_behaviour::show_help_prefix(bot_data),
        PrefixIsNotAChar => help_behaviour::show_help_prefix(bot_data),
        WrongArgumentForWatchList => help_behaviour::show_help_watchlist(bot_data),
        WrongArgumentForHistory => help_behaviour::show_help_history(bot_data),
        NotEnoughArgumentsForStatus | WrongArgumentsForStatus => {
            help_behaviour::show_help_status(bot_data)
        }
        NoArgumentForUnavailable | WrongArgumentForUnavailable => {
            help_behaviour::show_help_set_status_unavailable(bot_data)
        }
        NotEnoughArgumentsForWatched | WrongArgumentsForWatched => {
            help_behaviour::show_help_set_status_watched(bot_data)
        }
        NoArgumentsForShowMovie => help_behaviour::show_help_show_movie(bot_data),
        NoArgumentsForSearchMovie => help_behaviour::show_help_search_movie(bot_data),
        NoArgumentsForCreateVote => help_behaviour::show_help_create_vote(bot_data),
        WrongArgumentsForMovieLimit => help_behaviour::show_help_movie_limit(bot_data),
        WrongArgumentsForMovieVoteLimit => help_behaviour::show_help_movie_vote_limit(bot_data),
        WrongArgumentsForSendVoteWithUserId => help_behaviour::show_help_send_vote(bot_data),
        WrongArgumentForRandomMovieVote => help_behaviour::show_help_random_movie_vote(bot_data),
    }
}

#[cfg(test)]
mod tests {
    use super::{deserialize_bot_data, panic_payload_classification};

    #[test]
    fn deserialization_returns_client_construction_errors() {
        let input = r#"{"custom_prefix":".","movie_limit_per_user":10,"movie_vote_limit":2,"next_movie_id":0}"#;
        let mut deserializer = serde_json::Deserializer::from_str(input);

        let result = deserialize_bot_data(&mut deserializer, || Err::<_, _>("invalid client"));

        assert!(result.is_err());
    }

    #[test]
    fn panic_payload_classification_does_not_return_payload_content() {
        let secret = "discord-token-should-not-be-logged";

        assert_eq!(panic_payload_classification(&secret), "string_slice");
        assert_eq!(
            panic_payload_classification(&String::from(secret)),
            "string"
        );
        assert_eq!(panic_payload_classification(&42_u32), "non_string");
    }
}
