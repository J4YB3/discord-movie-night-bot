use discord::{model as Model};
use chrono::DateTime;
use std::collections::HashMap;
use itertools::Itertools;
use std::{fmt, cmp::Ordering, str::FromStr};
use crate::{COLOR_ERROR, COLOR_SUCCESS, COLOR_WARNING, COLOR_BOT, COLOR_INFORMATION};
use crate::general_behaviour::*;
use tmdb::{themoviedb::*};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum MovieStatus {
    NotWatched,
    Watched,
    Unavailable,
    Rewatch,
    Removed,
}

impl fmt::Display for MovieStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovieStatus::NotWatched => write!(f, "`NotWatched`"),
            MovieStatus::Watched => write!(f, "`Watched`"), 
            MovieStatus::Unavailable => write!(f, "`Unavailable`"), 
            MovieStatus::Rewatch => write!(f, "`Rewatch`"), 
            MovieStatus::Removed => write!(f, "`Removed`"), 
        }
    }
}

impl MovieStatus {
    /**
     * Returns the string slice containing the emoji corresponding to this status
     */
    fn get_emoji(&self) -> &str {
        match self {
            MovieStatus::NotWatched => ":white_large_square:",
            MovieStatus::Watched => ":white_check_mark:",
            MovieStatus::Unavailable => ":orange_square:",
            MovieStatus::Rewatch => ":recycle:",
            MovieStatus::Removed => ":red_square:"
        }
    }

    /**
     * Returns true if a movie with this status should be shown on the watch list
     */
    fn is_watch_list_status(&self) -> bool {
        match self {
            MovieStatus::NotWatched | MovieStatus::Rewatch | MovieStatus::Unavailable => true,
            _ => false,
        }
    }

    /**
     * Returns true if a movie with this status should be shown on the history
     */
    fn is_history_status(&self) -> bool {
        !self.is_watch_list_status()
    }
}

impl FromStr for MovieStatus {
    type Err = MovieStatusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "notwatched" => Self::NotWatched,
            "watched" => Self::Watched,
            "rewatch" => Self::Rewatch,
            "removed" => Self::Removed,
            "unavailable" => Self::Unavailable,
            _ => return Err(MovieStatusErr::UnknownStatus),
        })
    }
}

pub enum MovieStatusErr {
    UnknownStatus,
}

#[derive(Eq, Clone, Debug)]
pub struct WatchListEntry {
    movie_title: String,
    original_title: String,
    original_language: String,
    tmdb_id: u64,
    overview: String,
    poster_path: Option<String>,
    release_date: DateTime<chrono::FixedOffset>,
    genres: String,
    runtime: u32,
    budget: String,
    user: String,
    user_id: Model::UserId,
    status: MovieStatus,
    added_timestamp: DateTime<chrono::FixedOffset>,
    watched_or_removed_timestamp: Option<DateTime<chrono::FixedOffset>>,
}

impl Ord for WatchListEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.movie_title.cmp(&other.movie_title)
    }
}

impl PartialOrd for WatchListEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WatchListEntry {
    fn eq(&self, other: &Self) -> bool {
        self.movie_title == other.movie_title
    }
}

pub enum WaitingForReaction {
    AddMovie(Model::MessageId, WatchListEntry),
    NoWait
}

/**
 * Appends the poster path returned by tmdb search to the default tmdb poster directory
 */
fn get_movie_poster_link(poster_path: &str) -> String {
    String::from(format!("https://www.themoviedb.org/t/p/w220_and_h330_face{}", poster_path))
}

/**
 * Generates the poster link with the poster_path_option from the WatchListEntry
 * Automatically replaces the poster path with the no-image-available image if the path is None
 */
fn generate_poster_link(poster_path_option: &Option<String>) -> String {
    get_movie_poster_link(
        poster_path_option
        .as_ref()
        .unwrap_or(
            &get_no_image_available_url().to_string()
        )
    )
}

/**
 * Appends the tmdb id to the default tmdb watch overview link
 */
fn get_movie_link(tmdb_id: u64, watch_link: bool) -> String {
    String::from(format!("https://www.themoviedb.org/movie/{}{}", tmdb_id, if watch_link {"/watch"} else {""}))
}

/**
 * Shortens the overview of a movie to around 300 characters
 */
fn shorten_movie_description(overview: String) -> String {
    let mut i = 0;
    let mut short_overview = String::from("");
    for character in overview.chars() {
        i += 1;
        if i >= 300 && character.is_whitespace() {
            short_overview.push_str("...");
            return short_overview;
        }
        short_overview.push(character);
    }

    short_overview
}

/**
 * Receives the genres of a movie as vector and provides a string with the first 3 (or less) genres
 * as a comma separated String
 */
fn get_genres_formatted(genres: &Vec<tmdb::model::Genre>) -> String {
    return match genres.len() {
        0 => format!("Keine Genres vorhanden"),
        1 => format!("{}", genres[0].name),
        2 => format!("{}, {}", genres[0].name, genres[1].name),
        _ => {
            if genres.len() >= 3 {
                format!("{}, {}, {}", genres[0].name, genres[1].name, genres[2].name)
            } else {
                String::from("Fehler")
            }
        }
    };
}

/**
 * Searches a movie on TMDb and displays its information. 
 */
pub fn search_movie(bot_data: &mut crate::BotData, title_or_link: &str, add_movie: bool) {
    let message = bot_data.message.as_ref().expect("Passing message to search_movie function failed");

    enum SearchResult {
        Movie(tmdb::model::Movie),
        TryIMDBLink,
        Error
    }

    // Initiate the search
    let movie = if title_or_link.contains("imdb.com/") {
        if let Some(imdb_id) = parse_imdb_link_id(title_or_link.to_string()) {
            let result = bot_data.tmdb
            .find()
            .imdb_id(imdb_id.as_str())
            .execute()
            .unwrap();

            if result.movie_results.len() <= 0 {
                SearchResult::Error
            } else {
                SearchResult::Movie(
                    bot_data.tmdb
                    .fetch()
                    .id(result.movie_results[0].id)
                    .execute()
                    .unwrap()
                )
            }
        } else {
            SearchResult::Error
        }
    } else {
        let result = bot_data.tmdb
        .search()
        .title(title_or_link)
        .execute();
        
        if let Ok(result) = result {
            if result.total_results <= 0 {
                SearchResult::Error
            } else {
                let fetch_result = result.results[0].fetch(&bot_data.tmdb);

                if let Ok(result) = fetch_result {
                    SearchResult::Movie(result)
                } else {
                    SearchResult::TryIMDBLink
                }
            }
        } else {
            SearchResult::TryIMDBLink
        }
    };
    
    if let SearchResult::Movie(first_movie) = movie {
        // If the user wants to add this movie, check if it is already in the watch list
        if add_movie {
            if let Some(id) = find_id_by_tmdb_id(first_movie.id, &bot_data.watch_list) {
                // The movie that was just found is already in the watch list, so send a message
                // Also return from the function, because no new movie should be added in that case
                return send_movie_already_exists_message(bot_data, *id, first_movie.id);
            }
        }

        let new_entry = WatchListEntry {
            movie_title: first_movie.title.clone(),
            original_title: first_movie.original_title.clone(),
            original_language: first_movie.original_language.clone().to_uppercase(),
            overview: shorten_movie_description(first_movie.overview.clone().unwrap_or("Keine Beschreibung verfügbar.".to_string())),
            poster_path: first_movie.poster_path.clone(),
            tmdb_id: first_movie.id,
            genres: get_genres_formatted(&first_movie.genres),
            runtime: first_movie.runtime,
            budget: format_budget(first_movie.budget),
            release_date: parse_tmdb_release_date(first_movie.release_date.clone()).unwrap_or(message.timestamp),
            user: message.author.name.clone(),
            added_timestamp: message.timestamp,
            watched_or_removed_timestamp: None,
            status: MovieStatus::NotWatched,
            user_id: message.author.id,
        };

        let bot_response = send_movie_information_message(bot_data, &new_entry, true, add_movie);

        if add_movie {
            if let Ok(res_message) = bot_response {
                // Add the waiting for reaction enum entry to bot_data
                bot_data.wait_for_reaction = Some(WaitingForReaction::AddMovie(res_message.id, new_entry));

                // Add ✅ as reaction
                let _ = bot_data.bot.add_reaction(res_message.channel_id, res_message.id, Model::ReactionEmoji::Unicode("✅".to_string()));
                // Add ❎ as reaction
                let _ = bot_data.bot.add_reaction(res_message.channel_id, res_message.id, Model::ReactionEmoji::Unicode("❎".to_string()));
            }
        }
    } else if let SearchResult::TryIMDBLink = movie {
        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
            .title("Fehler bei der TMDb Suche")
            .description(
                format!("Leider ist bei der TMDb Suche ein Fehler aufgetreten. Bitte versuche es erneut mit einem IMDb Link :kissing_heart:")
                .as_str()
            )
            .color(COLOR_ERROR)
        );
    } else {
        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
            .title("Keine Filme gefunden")
            .description(
                format!("Leider konnten keine Filme mit diesem Titel gefunden werden, oder der angegebene Link war fehlerhaft :cry:")
                .as_str()
            )
            .color(COLOR_ERROR)
        );
    }
}

/**
 * Analyses the reaction to the search result of the add_movie command and adds or discards the movie
 */
pub fn add_movie_by_reaction(bot_data: &mut crate::BotData, reaction: discord::model::Reaction) {
    if let Some(waiting_for_reaction_enum) = bot_data.wait_for_reaction.as_ref() {
        if let WaitingForReaction::AddMovie(_, new_entry) = waiting_for_reaction_enum {
            if reaction_emoji_equals(reaction.emoji, "✅".to_string()) {
                let copied_entry = WatchListEntry {
                    movie_title: new_entry.movie_title.clone(),
                    original_title: new_entry.original_title.clone(),
                    original_language: new_entry.original_language.clone(),
                    overview: new_entry.overview.clone(),
                    poster_path: new_entry.poster_path.clone(),
                    budget: new_entry.budget.clone(),
                    genres: new_entry.genres.clone(),
                    user: new_entry.user.clone(),
                    status: new_entry.status.clone(),
                    ..*new_entry
                };
                bot_data.watch_list.insert(bot_data.next_movie_id, copied_entry);
    
                bot_data.next_movie_id += 1;

                let _ = bot_data.bot.send_embed(
                    reaction.channel_id,
                    "",
                    |embed| embed
                    .title(format!("{} wurde erfolgreich hinzugefügt", new_entry.movie_title).as_str())
                    .thumbnail(generate_poster_link(&new_entry.poster_path).as_str())
                    .fields(|fields| fields
                        .field("ID", format!("`{:0>4}`", bot_data.next_movie_id - 1).as_str(), true)
                        .field("Status", new_entry.status.get_emoji(), true)
                        .field("Hinzugefügt am", timestamp_to_string(&new_entry.added_timestamp, true).as_str(), true)
                        .field("Hinzugefügt von", format!("<@{}>", new_entry.user_id).as_str(), true)
                    )
                    .color(COLOR_SUCCESS)
                );
            } else {
                let _ = bot_data.bot.send_embed(
                    reaction.channel_id,
                    "",
                    |embed| embed
                    .title("Hinzufügen abgebrochen.")
                    .description("Nicht der richtige Film? Versuche das Hinzufügen mit einem IMDb Link.")
                    .color(COLOR_INFORMATION)
                );
            }
            
            bot_data.wait_for_reaction = None;
        }
    }
}

/**
 * Sends an embedded message that the movie was already added by someone
 */
fn send_movie_already_exists_message(bot_data: &crate::BotData, id: u32, tmdb_id: u64) {
    let message = bot_data.message.as_ref().expect("Passing message to send_movie_already_exists_message function failed");
    let previous_entry = bot_data.watch_list.get(&id).expect("Accessing the watch list has failed inside the send_movie_already_exists_message function.");

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed
        .title(format!("{}", previous_entry.movie_title).as_str())
        .url(get_movie_link(tmdb_id, false).as_str())
        .thumbnail(generate_poster_link(&previous_entry.poster_path).as_str())
        .description(
            format!("**{}** hat diesen Film bereits am *{}* hinzugefügt.\nFalls du einen anderen Film meinst versuche das Hinzufügen durch einen IMDb Link.", 
                previous_entry.user,
                timestamp_to_string(&previous_entry.added_timestamp, true),
            )
            .as_str()
        )
        .color(COLOR_ERROR)
    );
}

/**
 * Takes a movie entry and sends an embedded message with all information of the movie
 */
fn send_movie_information_message(bot_data: &crate::BotData, movie_entry: &WatchListEntry, new_movie: bool, ask_confirmation: bool) -> Result<Model::Message, discord::Error> {
    let message = bot_data.message.as_ref().expect("Passing message to send_movie_information_message function failed.");

    if new_movie {
        bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
            .title(format!("{}", movie_entry.movie_title).as_str())
            .url(
                get_movie_link(movie_entry.tmdb_id, false).as_str()
            )
            .description(
                movie_entry.overview.clone().as_str()
            )
            .image(generate_poster_link(&movie_entry.poster_path).as_str())
            .color(COLOR_SUCCESS)
            .fields(|fields| fields
                .field("Originaltitel", movie_entry.original_title.as_str(), true)
                .field("Originalsprache", movie_entry.original_language.as_str(), true)
                .field("Erschienen", timestamp_to_string(&movie_entry.release_date, false).as_str(), true)
                .field("Genres", movie_entry.genres.as_str(), true)
                .field("Dauer", format!("{} min", movie_entry.runtime).as_str(), true)
                .field("Budget", movie_entry.budget.as_str(), true)
            )
            .footer(|footer| footer
                .text(format!("{}", if ask_confirmation {"Meintest du diesen Film?"} else {""}).as_str())
            )
            .thumbnail(get_tmdb_attribution_icon_url())
        )
    } else {
        bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
            .title(format!("{}", movie_entry.movie_title).as_str())
            .url(
                get_movie_link(movie_entry.tmdb_id, false).as_str()
            )
            .description(
                movie_entry.overview.clone().as_str()
            )
            .image(generate_poster_link(&movie_entry.poster_path).as_str())
            .color(COLOR_SUCCESS)
            .fields(|fields| fields
                .field("Originaltitel", movie_entry.original_title.as_str(), true)
                .field("Originalsprache", movie_entry.original_language.as_str(), true)
                .field("Erschienen", timestamp_to_string(&movie_entry.release_date, false).as_str(), true)
                .field("Genres", movie_entry.genres.as_str(), true)
                .field("Dauer", format!("{} min", movie_entry.runtime).as_str(), true)
                .field("Budget", movie_entry.budget.as_str(), true)
                .field("Hinzugefügt von", format!("<@{}>", movie_entry.user_id).as_str(), true)
                .field("Hinzugefügt am", timestamp_to_string(&movie_entry.added_timestamp, true).as_str(), true)
                .field("Status", format!("{}", movie_entry.status.get_emoji()).as_str(), true)
            )
            .thumbnail(get_tmdb_attribution_icon_url())
        )
    }
}

/**
 * Removes a movie from the watch list when given a title. Checks if the user has administrator rights.
 * If so he is allowed to remove any movie by any user. Normal users are only allowed to remove their own
 * movies.
 */
pub fn remove_movie_by_title(bot_data: &mut crate::BotData, title: &str) {
    let possible_id = get_movie_id_in_watch_list(title, &bot_data.watch_list);

    match possible_id {
        Some(id) => remove_movie_by_id(bot_data, id),
        None => {
            let message = bot_data.message.as_ref().expect("Passing message to remove_movie_by_title function failed.");
            let _ = bot_data.bot.send_embed(
                message.channel_id,
                "",
                |embed| embed.description(
                    format!("Ein Film mit dem Titel **{}** konnte nicht in der Liste gefunden werden.", title).as_str(),
                )
                .color(COLOR_ERROR)
            );
        }
    }
}

/**
 * Removes a movie by its ID. Checks if the user has administrator rights. If so he is allowed 
 * to remove any movie by any user. Normal users are only allowed to remove their own movies.
 */
pub fn remove_movie_by_id(bot_data: &mut crate::BotData, id: u32) {
    let message = bot_data.message.as_ref().expect("Passing message to remove_movie_by_title function failed.");

    let user_is_admin: bool = is_user_administrator(bot_data, message.author.id);

    let movie = bot_data.watch_list.get(&id);
    match movie {
        Some(watch_list_entry) => {
            if watch_list_entry.user == message.author.name {
                let _ = bot_data.bot.send_embed(
                    message.channel_id,
                    "",
                    |embed| embed.description(
                        format!("Film entfernt. Senk ju for trevelling wis Deutsche Bahn. Ach ne das war falsch :sweat_smile:
                        `{:0>4}` {} **{}** | hinzugefügt am *{}* von dir selbst", 
                            id, 
                            watch_list_entry.status.get_emoji(), 
                            watch_list_entry.movie_title,
                            timestamp_to_string(&watch_list_entry.added_timestamp, true)
                        ).as_str(),
                    )
                    .color(COLOR_WARNING)
                );
                let _ = bot_data.watch_list.remove(&id);
            } else if user_is_admin {
                let _ = bot_data.bot.send_embed(
                    message.channel_id,
                    "",
                    |embed| embed.description(
                        format!("Film entfernt. Bitte beehren Sie uns bald wieder.\n`{:0>4}` {} **{}** | hinzugefügt am *{}* von <@{}>", 
                            id, 
                            watch_list_entry.status.get_emoji(), 
                            watch_list_entry.movie_title,
                            timestamp_to_string(&watch_list_entry.added_timestamp, true),
                            watch_list_entry.user_id
                        ).as_str(),
                    )
                    .color(COLOR_WARNING)
                );
                let _ = bot_data.watch_list.remove(&id);
            } else {
                let _ = bot_data.bot.send_embed(
                    message.channel_id,
                    "",
                    |embed| embed.description(
                        format!("Du hat nicht genügend Rechte um den Film zu entfernen. Klagen bitte an das Verfassungsgericht.
                        `{:0>4}` {} **{}** | hinzugefügt am *{}* von <@{}>", 
                            id, 
                            watch_list_entry.status.get_emoji(), 
                            watch_list_entry.movie_title,
                            timestamp_to_string(&watch_list_entry.added_timestamp, true),
                            watch_list_entry.user_id
                        ).as_str(),
                    )
                    .color(COLOR_WARNING)
                );
            }
            
        },
        None => {
            let _ = bot_data.bot.send_embed(
                message.channel_id,
                "",
                |embed| embed.description(
                    format!("Ein Film mit der ID `{:0>4}` konnte nicht gefunden werden.", id).as_str(),
                )
                .color(COLOR_ERROR)
            );
        }
    }
}

/**
 * Formats the watch list hash map and sends it as an embedded message
 */
pub fn show_watch_list(bot_data: &crate::BotData, order: String) {
    let message = bot_data.message.as_ref().expect("Passing message to show_watch_list function failed.");

    let mut watch_list_string: String = String::new();
    let history_count = count_watch_list_movies(bot_data);
    if history_count == 1 {
        watch_list_string += format!("Es ist zur Zeit **{}** Film auf der Liste\n", history_count).as_str();
    } else {
        watch_list_string += format!("Es sind zur Zeit **{}** Filme auf der Liste\n", history_count).as_str();
    }

    if history_count > 0 {
        // If the ordering by user is demanded
        if order == "user" {
            watch_list_string += format!("Die Filme werden geordnet nach dem Nutzer angezeigt, welcher sie hinzugefügt hat.\n\n").as_str();

            let user_movies = create_user_sorted_watch_list(bot_data);

            // For every user
            for (user, entry_vector) in user_movies.iter().sorted() {
                watch_list_string += format!("Filme hinzugefügt von **{}**\n", user).as_str();
                
                // For every movie entry
                for (id, entry) in entry_vector {
                    watch_list_string += format!("`{:0>4}`", id.to_string()).as_str();
                    watch_list_string += format!(" {} **{}** | hinzugefügt am *{}*\n", entry.status.get_emoji(), entry.movie_title, timestamp_to_string(&entry.added_timestamp, false)).as_str();
                }

                watch_list_string += "\n";
            }
        } 
        // If the ordering should be random
        else {
            watch_list_string += format!("Die Filme werden in zufälliger Reihenfolge angezeigt.\n\n").as_str();

            // For every movie entry
            for (id, entry) in bot_data.watch_list.iter() {
                if entry.status.is_watch_list_status() {
                    watch_list_string += format!("`{:0>4}`", id.to_string()).as_str();
                    watch_list_string += format!(" {} **{}** | hinzugefügt von **{}** am *{}*\n", entry.status.get_emoji(), entry.movie_title, entry.user, timestamp_to_string(&entry.added_timestamp, false)).as_str();
                }
            }
        }
    }
    
    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title("Watch list").description(watch_list_string.as_str()).color(COLOR_BOT)
    );
}

/**
 * Collects all movie entries from the watch list that have a watch list status
 * and returns them in a new HashMap that contains the user as key.
 */
fn create_user_sorted_watch_list(bot_data: &crate::BotData) -> HashMap<&String, Vec<(u32, &WatchListEntry)>> {
    // Create a hashmap that stores the user as key and a tuple containing the id and the watch list entry of every movie
    let mut user_movies: HashMap<&String, Vec<(u32, &WatchListEntry)>> = HashMap::new();
    for (id, entry) in bot_data.watch_list.iter().sorted() {
        if entry.status.is_watch_list_status() {
            // Append the id, entry tuple to the user movie vector
            if let Some(vector) = user_movies.get_mut(&entry.user) {
                vector.push((*id, &entry));
            }
            // if the user got no movies yet create the vector and add the first tuple
            else {
                user_movies.insert(&entry.user, vec![(*id, entry)]);
            }
        }
    }

    user_movies
}

/**
 * Counts all movies from the list with watch list status and returns the count
 */
fn count_watch_list_movies(bot_data: &crate::BotData) -> u32 {
    let mut count: u32 = 0;

    for (_, entry) in bot_data.watch_list.iter() {
        if entry.status.is_watch_list_status() {
            count += 1;
        }
    }

    count
}

/**
 * Formats the watch list hash map as a movie history and sends it as an embedded message
 */
pub fn show_history(bot_data: &crate::BotData, order: String) {
    let message = bot_data.message.as_ref().expect("Passing message to show_history function failed.");

    let mut history_string: String = String::new();
    let history_count = count_history_movies(bot_data);
    if history_count == 1 {
        history_string += format!("Es ist zur Zeit **{}** Film im Verlauf\n", history_count).as_str();
    } else {
        history_string += format!("Es sind zur Zeit **{}** Filme im Verlauf\n", history_count).as_str();
    }

    if history_count > 0 {
        // If the ordering by user is demanded
        if order == "user" {
            history_string += format!("Die Filme werden geordnet nach dem Nutzer angezeigt, welcher sie hinzugefügt hat.\n\n").as_str();

            let user_movies = create_user_sorted_history(bot_data);

            // For every user
            for (user, entry_vector) in user_movies.iter().sorted() {
                history_string += format!("Filme hinzugefügt von **{}**\n", user).as_str();
                
                let mut time_sorted_vector = entry_vector.clone();
                
                time_sorted_vector.sort_by( 
                    |a, b| 
                    a.1.watched_or_removed_timestamp
                        .expect("Movie did not have a watched_or_removed timestamp in show_history")
                        .cmp(
                            &b.1.watched_or_removed_timestamp
                            .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                        )
                    );

                // For every movie entry
                for (id, entry) in time_sorted_vector {
                    history_string += format!("`{:0>4}`", id.to_string()).as_str();
                    
                    history_string += format!(" {} **{}** | {} am *{}*\n", 
                        entry.status.get_emoji(), 
                        entry.movie_title, 
                        if entry.status == MovieStatus::Watched {"geschaut"} else {"entfernt"},
                        timestamp_to_string(
                            &entry.watched_or_removed_timestamp
                                .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                            , false)
                        ).as_str();
                }

                history_string += "\n";
            }
        } 
        // If the ordering should be by date
        else {
            history_string += format!("Die Filme werden in zeitlicher Reihenfolge angezeigt.\n\n").as_str();

            let mut time_sorted_vector = bot_data.watch_list.clone().into_iter()
                .filter(|(_id, entry)| entry.status.is_history_status())
                .map(|(id, entry)| (id, entry))
                .collect::<Vec<(u32, WatchListEntry)>>();

            time_sorted_vector.sort_by( 
                |a, b| 
                a.1.watched_or_removed_timestamp
                    .expect("Movie did not have a watched_or_removed timestamp in show_history")
                    .cmp(
                        &b.1.watched_or_removed_timestamp
                        .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                    )
                );

            // For every movie entry
            for (id, entry) in time_sorted_vector {
                if entry.status.is_history_status() {
                    history_string += format!("`{:0>4}`", id.to_string()).as_str();
                    history_string += format!(" {} **{}** | hinzugefügt von **{}**, {} am *{}*\n", 
                        entry.status.get_emoji(), 
                        entry.movie_title, 
                        entry.user, 
                        if entry.status == MovieStatus::Watched {"geschaut"} else {"entfernt"},
                        timestamp_to_string(
                            &entry.watched_or_removed_timestamp
                            .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                            , false
                        )
                    ).as_str();
                }
            }
        }
    }
    
    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title("History").description(history_string.as_str()).color(COLOR_BOT)
    );
}

/**
 * Collects all movie entries from the watch list that have a history status
 * and returns them in a new HashMap that contains the user as key.
 */
fn create_user_sorted_history(bot_data: &crate::BotData) -> HashMap<&String, Vec<(u32, &WatchListEntry)>> {
    // Create a hashmap that stores the user as key and a tuple containing the id and the watch list entry of every movie
    let mut user_movies: HashMap<&String, Vec<(u32, &WatchListEntry)>> = HashMap::new();
    for (id, entry) in bot_data.watch_list.iter().sorted() {
        if entry.status.is_history_status() {
            if entry.watched_or_removed_timestamp.is_none() {
                continue;
            }

            // Append the id, entry tuple to the user movie vector
            if let Some(vector) = user_movies.get_mut(&entry.user) {
                vector.push((*id, &entry));
            }
            // if the user got no movies yet create the vector and add the first tuple
            else {
                user_movies.insert(&entry.user, vec![(*id, entry)]);
            }
        }
    }

    user_movies
}

/**
 * Counts all movies from the list with history status and returns the count
 */
fn count_history_movies(bot_data: &crate::BotData) -> u32 {
    let mut count: u32 = 0;

    for (_, entry) in bot_data.watch_list.iter() {
        if entry.status.is_history_status() {
            count += 1;
        }
    }

    count
}

/** 
 * Sets the status of a movie given by id. Only administrator users are allowed to change status of a movie.
 * If the given status is either watched or removed the timestamp for removal/watched gets set
 */
pub fn set_status(bot_data: &mut crate::BotData, id: u32, status: String) {
    let message = bot_data.message.as_ref().expect("Passing message to set_status function failed.");

    let user_is_admin: bool = is_user_administrator(bot_data, message.author.id);

    let status_result = MovieStatus::from_str(status.as_str());

    if let Ok(new_status) = status_result {
        let movie = bot_data.watch_list.get(&id);
        match movie {
            Some(watch_list_entry) => {
                let original_movie_status = watch_list_entry.status.clone();

                // Only allow change of status if the user has the admin role
                if user_is_admin {
                    let mut updated_entry = WatchListEntry {
                        status: new_status.clone(),
                        movie_title: watch_list_entry.movie_title.clone(),
                        original_title: watch_list_entry.original_title.clone(),
                        original_language: watch_list_entry.original_language.clone(),
                        user: watch_list_entry.user.clone(),
                        overview: watch_list_entry.overview.clone(),
                        poster_path: watch_list_entry.poster_path.clone(),
                        budget: watch_list_entry.budget.clone(),
                        genres: watch_list_entry.genres.clone(),
                        ..*watch_list_entry
                    };

                    if new_status.is_history_status() {
                        updated_entry.watched_or_removed_timestamp = Some(message.timestamp);
                    }

                    let _ = bot_data.bot.send_embed(
                        message.channel_id,
                        "",
                        |embed| embed.title(format!("Status des Films `{:0>4}` **{}** hinzugefügt von {} wurde geändert.", id, updated_entry.movie_title, updated_entry.user).as_str())
                        .description(
                            format!("
                            **von** {} ({}) **zu** {} ({})", 
                                original_movie_status.get_emoji(), 
                                original_movie_status, 
                                updated_entry.status.get_emoji(), 
                                updated_entry.status,
                            ).as_str()
                        )
                        .color(COLOR_SUCCESS)
                    );
                    let _ = bot_data.watch_list.insert(id, updated_entry);
                // If the user is not an admin and did not add the movie himself he is not permitted to change it
                } else {
                    let _ = bot_data.bot.send_embed(
                        message.channel_id,
                        "",
                        |embed| embed.description(
                            format!("Du hat nicht genügend Rechte um den Status des Films zu ändern. Klagen bitte an das Verfassungsgericht.
                            `{:0>4}` {} **{}** | hinzugefügt am *{}* von <@{}>", 
                                id, 
                                watch_list_entry.status.get_emoji(), 
                                watch_list_entry.movie_title,
                                timestamp_to_string(&watch_list_entry.added_timestamp, true),
                                watch_list_entry.user_id
                            ).as_str(),
                        )
                        .color(COLOR_WARNING)
                    );
                }
            },
            None => {
                let _ = bot_data.bot.send_embed(
                    message.channel_id,
                    "",
                    |embed| embed.description(
                        format!("Ein Film mit der ID `{:0>4}` konnte nicht gefunden werden.", id).as_str(),
                    )
                    .color(COLOR_ERROR)
                );
            }
        }
    } else {
        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed.title("Falscher Status")
                .description(format!("Ein Status mit dem Namen {} existiert nicht. Folgende Status sind möglich:
                `Watched`, `NotWatched`, `Removed`, `Rewatch`, `Unavailable`", status).as_str())
                .color(COLOR_BOT)
        );
    }
}

/** 
 * If a custom date is given in the format DD.MM.YYYY converts it to a timestamp, saves this to the message
 * and calls the set_status function with "watched" as status
 */
pub fn set_status_watched(bot_data: &mut crate::BotData, id: u32, date: String) {
    if date.is_empty() {
        set_status(bot_data, id, "Watched".to_string());
    } else {
        let message = bot_data.message.as_ref().expect("Passing message to set_status function failed.");
        let date_with_utc = date.clone() + " 12:00:00.000 +0000";
        if let Ok(datetime) = chrono::DateTime::parse_from_str(date_with_utc.as_str(), "%d.%m.%Y %H:%M:%S%.3f %z") {
            let user_is_admin: bool = is_user_administrator(bot_data, message.author.id);

            let new_status = MovieStatus::Watched;
            
            let movie = bot_data.watch_list.get(&id);

            match movie {
                Some(watch_list_entry) => {
                    let original_movie_status = watch_list_entry.status.clone();

                    // Only allow change of status if the user has the admin role
                    if user_is_admin {
                        let updated_entry = WatchListEntry {
                            status: new_status.clone(),
                            movie_title: watch_list_entry.movie_title.clone(),
                            original_title: watch_list_entry.original_title.clone(),
                            original_language: watch_list_entry.original_language.clone(),
                            user: watch_list_entry.user.clone(),
                            watched_or_removed_timestamp: Some(datetime),
                            overview: watch_list_entry.overview.clone(),
                            poster_path: watch_list_entry.poster_path.clone(),
                            budget: watch_list_entry.budget.clone(),
                            genres: watch_list_entry.genres.clone(),
                            ..*watch_list_entry
                        };

                        let _ = bot_data.bot.send_embed(
                            message.channel_id,
                            "",
                            |embed| embed.title(format!("Status des Films `{:0>4}` **{}** hinzugefügt von {} wurde geändert.", id, updated_entry.movie_title, updated_entry.user).as_str())
                            .description(
                                format!("
                                **von** {} ({}) **zu** {} ({})", 
                                    original_movie_status.get_emoji(), 
                                    original_movie_status, 
                                    updated_entry.status.get_emoji(), 
                                    updated_entry.status,
                                ).as_str()
                            )
                            .color(COLOR_SUCCESS)
                        );
                        let _ = bot_data.watch_list.insert(id, updated_entry);
                    // If the user is not an admin and did not add the movie himself he is not permitted to change it
                    } else {
                        let _ = bot_data.bot.send_embed(
                            message.channel_id,
                            "",
                            |embed| embed.description(
                                format!("Du hat nicht genügend Rechte um den Status des Films zu ändern. Klagen bitte an das Verfassungsgericht.
                                `{:0>4}` {} **{}** | hinzugefügt am *{}* von <@{}>", 
                                    id, 
                                    watch_list_entry.status.get_emoji(), 
                                    watch_list_entry.movie_title,
                                    timestamp_to_string(&watch_list_entry.added_timestamp, true),
                                    watch_list_entry.user_id
                                ).as_str(),
                            )
                            .color(COLOR_WARNING)
                        );
                    }
                },
                None => {
                    let _ = bot_data.bot.send_embed(
                        message.channel_id,
                        "",
                        |embed| embed.description(
                            format!("Ein Film mit der ID `{:0>4}` konnte nicht gefunden werden.", id).as_str(),
                        )
                        .color(COLOR_ERROR)
                    );
                }
            }
        } else {
            let _ = bot_data.bot.send_embed(
                message.channel_id,
                "",
                |embed| embed.description(
                    format!("Das Datum '{}' hatte leider das falsche Format. Bitte stelle sicher, dass das Datum im Format TT.MM.JJJJ vorliegt.", date).as_str(),
                )
                .color(COLOR_ERROR)
            );
        }

    }
}

/**
 * Shows the movie information to the movie id or an error message if the id does not exist
 */
pub fn show_movie_by_id(bot_data: &crate::BotData, id: u32) {
    if let Some(entry) = bot_data.watch_list.get(&id) {
        let _ = send_movie_information_message(bot_data, entry, false, false);
    } else {
        let _ = bot_data.bot.send_embed(
            bot_data.message.as_ref().expect("Passing of message to show_movie_by_id function failed.").channel_id,
            "",
            |embed| embed
            .title("Film existiert nicht")
            .description("Ein Film mit dieser ID existiert weder in der Watch list noch in der History.")
            .color(COLOR_ERROR)
        );
    }
}

/**
 * Shows the movie information to the movie title or an error message if th etitle does not exist
 */
pub fn show_movie_by_title(bot_data: &crate::BotData, title: String) {
    if let Some(id) = get_movie_id_in_watch_list(title.as_str(), &bot_data.watch_list) {
        return show_movie_by_id(bot_data, id);
    } else {
        let _ = bot_data.bot.send_embed(
            bot_data.message.as_ref().expect("Passing of message to show_movie_by_title function failed.").channel_id,
            "",
            |embed| embed
            .title("Film existiert nicht")
            .description("Ein Film mit diesem Namen existiert weder in der Watch list noch in der History.")
            .color(COLOR_ERROR)
        );
    }
}

/**
 * Checks all roles of the user for admin permissions and returns true if the user has at least one
 * role with those permissions
 */
pub fn is_user_administrator(bot_data: &crate::BotData, user_id: Model::UserId) -> bool {
    let author_role_ids = bot_data.bot.get_member(bot_data.server_id, user_id).expect("Retrieval of author user failed.").roles;

    for role in &bot_data.server_roles {
        if author_role_ids.contains(&role.id) {
            if is_role_administrator(role) {
                return true
            }
        }
    }

    return false
}

/**
 * Returns true if the given role has administrator permissions
 */
fn is_role_administrator(role: &Model::Role) -> bool {
    let admin_permissions = Model::permissions::Permissions::ADMINISTRATOR;
    role.permissions.contains(admin_permissions)
}

/**
 * Returns the watch list id of the movie if the movie was found
 */
fn get_movie_id_in_watch_list(title: &str, watch_list: &HashMap<u32, WatchListEntry>) -> Option<u32> {
    for (id, entry) in watch_list {
        if entry.movie_title.to_lowercase() == title.to_lowercase() || entry.original_title.to_lowercase() == title.to_lowercase() {
            return Some(*id);
        }
    }
    None
}

/**
 * Returns the watch list id of the movie if the movie was found
 */
fn find_id_by_tmdb_id(tmdb_id: u64, watch_list: &HashMap<u32, WatchListEntry>) -> Option<&u32> {
    for (id, entry) in watch_list {
        if entry.tmdb_id == tmdb_id {
            return Some(id);
        }
    }
    None
}