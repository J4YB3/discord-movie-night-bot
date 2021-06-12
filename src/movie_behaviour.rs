use discord::{model as Model};
use chrono::DateTime;
use std::collections::HashMap;
use itertools::Itertools;
use std::{fmt, cmp::Ordering, str::FromStr};
use crate::{COLOR_ERROR, COLOR_SUCCESS, COLOR_BOT, COLOR_INFORMATION};
use crate::general_behaviour::*;
use crate::send_message;
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
    pub fn get_emoji(&self) -> &str {
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
    pub movie: Movie,
    pub user: String,
    pub user_id: Model::UserId,
    pub status: MovieStatus,
    pub added_timestamp: DateTime<chrono::FixedOffset>,
    pub watched_or_removed_timestamp: Option<DateTime<chrono::FixedOffset>>,
}

impl Ord for WatchListEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.movie.cmp(&other.movie)
    }
}

impl PartialOrd for WatchListEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WatchListEntry {
    fn eq(&self, other: &Self) -> bool {
        self.movie == other.movie
    }
}

#[derive(Eq, Clone, Debug)]
pub struct Movie {
    pub movie_title: String,
    pub original_title: String,
    pub original_language: String,
    pub tmdb_id: u64,
    pub overview: String,
    pub poster_path: Option<String>,
    pub release_date: DateTime<chrono::FixedOffset>,
    pub genres: String,
    pub runtime: u32,
    pub budget: String,
}

impl Ord for Movie {
    fn cmp(&self, other: &Self) -> Ordering {
        self.movie_title.cmp(&other.movie_title)
    }
}

impl PartialOrd for Movie {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Movie {
    fn eq(&self, other: &Self) -> bool {
        self.movie_title == other.movie_title
    }
}

/**
 * Appends the poster path returned by tmdb search to the default tmdb poster directory
 */
pub fn get_movie_poster_link(poster_path: &str) -> String {
    String::from(format!("https://www.themoviedb.org/t/p/w220_and_h330_face{}", poster_path))
}

/**
 * Generates the poster link with the poster_path_option from the WatchListEntry
 * Automatically replaces the poster path with the no-image-available image if the path is None
 */
pub fn generate_poster_link(poster_path_option: &Option<String>) -> String {
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
pub fn get_movie_link(tmdb_id: u64, watch_link: bool) -> String {
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
                return send_message::movie_already_exists(bot_data, *id, first_movie.id);
            } else {
                // Check if the user has already added up to the maximum limit of movies
                if user_has_too_many_movies(bot_data, message.author.id) {
                    // If that is the case, return because the movie should not be added
                    return send_message::user_has_too_many_movies_error(bot_data);
                }
            }
        }

        let new_movie = Movie {
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
        };

        let new_entry = WatchListEntry {
            movie: new_movie,
            user: message.author.name.clone(),
            added_timestamp: message.timestamp,
            watched_or_removed_timestamp: None,
            status: MovieStatus::NotWatched,
            user_id: message.author.id,
        };

        let bot_response = send_message::movie_information(bot_data, &new_entry, true, add_movie);

        if add_movie {
            if let Ok(res_message) = bot_response {
                // Add the waiting for reaction enum entry to bot_data
                bot_data.wait_for_reaction.push(WaitingForReaction::AddMovie(res_message.id, new_entry));

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
 * Checks if the user has more than the allowed limit of movies in
 * the watch list and returns the result as bool
 */
fn user_has_too_many_movies(bot_data: &crate::BotData, user_id: discord::model::UserId) -> bool {
    let mut movie_count = 0;

    // For every entry in the watch list, check if the user_id matches and the movie
    // counts as watch list movie (not history)
    for (_, movie_entry) in bot_data.watch_list.iter() {
        if movie_entry.user_id == user_id && movie_entry.status.is_watch_list_status() {
            movie_count += 1;
        }
    }

    if movie_count >= bot_data.movie_limit_per_user {
        return true;
    }

    false
}

/**
 * Analyses the reaction to the search result of the add_movie command and adds or discards the movie
 */
pub fn add_movie_by_reaction(bot_data: &mut crate::BotData, reaction: &discord::model::Reaction, new_entry: &WatchListEntry) {
    if reaction_emoji_equals(&reaction.emoji, "✅".to_string()) {
        let copied_entry = WatchListEntry {
            movie: new_entry.movie.clone(),
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
            .title(format!("{} wurde erfolgreich hinzugefügt", new_entry.movie.movie_title).as_str())
            .thumbnail(generate_poster_link(&new_entry.movie.poster_path).as_str())
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
            send_message::movie_title_not_found_error(bot_data, title.to_string());
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
                send_message::movie_removed_successfully(bot_data);
                let _ = bot_data.watch_list.remove(&id);
            } else if user_is_admin {
                send_message::movie_removed_successfully(bot_data);
                let _ = bot_data.watch_list.remove(&id);
            } else {
                send_message::insufficient_permissions_error(bot_data);
            }
            
        },
        None => {
            send_message::movie_id_not_found_error(bot_data, &id);
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

            watch_list_string += generate_user_sorted_watch_list_description(bot_data).as_str();
        } 
        // If the ordering should be random
        else {
            watch_list_string += format!("Die Filme werden in zufälliger Reihenfolge angezeigt.\n\n").as_str();

            watch_list_string += generate_random_sorted_watch_list_description(bot_data).as_str();
        }
    }
    
    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title("Filmliste").description(watch_list_string.as_str()).color(COLOR_BOT)
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
 * Generates the description text for the watch list sorted by user
 */
fn generate_user_sorted_watch_list_description(bot_data: &crate::BotData) -> String {
    let user_movies = create_user_sorted_watch_list(bot_data);
    let mut watch_list_string = String::new();

    // For every user
    for (user, entry_vector) in user_movies.iter().sorted() {
        watch_list_string += format!("Hinzugefügt von **{}**\n\n", user).as_str();
        
        // For every movie entry
        for (id, entry) in entry_vector {
            watch_list_string += format!(" {} [**{}**]({})\n> `{:0>4}` | hinzugefügt am {}\n\n", 
                entry.status.get_emoji(), 
                entry.movie.movie_title, 
                get_movie_link(entry.movie.tmdb_id, false), 
                id.to_string(), 
                timestamp_to_string(&entry.added_timestamp, false)
            ).as_str();
        }

        watch_list_string += "\n";
    }

    watch_list_string
}

/**
 * Generates the description text for the randomly sorted watch list
 */
fn generate_random_sorted_watch_list_description(bot_data: &crate::BotData) -> String {
    let mut watch_list_string = String::new();

    // For every movie entry
    for (id, entry) in bot_data.watch_list.iter() {
        if entry.status.is_watch_list_status() {
            watch_list_string += format!(" {} [**{}**]({})\n> `{:0>4}` | hinzugefügt von **{}** am {}\n\n", 
                entry.status.get_emoji(), 
                entry.movie.movie_title, 
                get_movie_link(entry.movie.tmdb_id, false), 
                id.to_string(), 
                entry.user, 
                timestamp_to_string(&entry.added_timestamp, false)
            ).as_str();
        }
    }

    watch_list_string
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

            history_string += generate_user_sorted_history_description(bot_data).as_str();
        } 
        // If the ordering should be by date
        else {
            history_string += format!("Die Filme werden in zeitlicher Reihenfolge angezeigt.\n\n").as_str();

            history_string += generate_random_sorted_history_description(bot_data).as_str();
        }
    }
    
    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title("Verlauf").description(history_string.as_str()).color(COLOR_BOT)
    );
}

/**
 * Generates the description for the user sorted history
 */
fn generate_user_sorted_history_description(bot_data: &crate::BotData) -> String {
    let user_movies = create_user_sorted_history(bot_data);
    let mut history_string = String::new();

    // For every user
    for (user, entry_vector) in user_movies.iter().sorted() {
        history_string += format!("Hinzugefügt von **{}**\n\n", user).as_str();
        
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
            history_string += format!(" {} [**{}**]({})\n> `{:0>4}` | {} am {}\n\n", 
                entry.status.get_emoji(), 
                entry.movie.movie_title, 
                get_movie_link(entry.movie.tmdb_id, false),
                id.to_string(),
                if entry.status == MovieStatus::Watched {"geschaut"} else {"entfernt"},
                timestamp_to_string(
                    &entry.watched_or_removed_timestamp
                        .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                    , false)
                ).as_str();
        }

        history_string += "\n";
    }

    history_string
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

fn generate_random_sorted_history_description(bot_data: &crate::BotData) -> String {
    let mut history_string = String::new();

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
            history_string += format!(" {} [**{}**]({})\n> `{:0>4}` | hinzugefügt von **{}**, {} am {}\n\n", 
                entry.status.get_emoji(), 
                entry.movie.movie_title, 
                get_movie_link(entry.movie.tmdb_id, false),
                id.to_string(),
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

    history_string
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
                // Only allow change of status if the user has the admin role
                if user_is_admin {
                    let mut updated_entry = WatchListEntry {
                        movie: watch_list_entry.movie.clone(),
                        status: new_status.clone(),
                        user: watch_list_entry.user.clone(),
                        ..*watch_list_entry
                    };

                    if new_status.is_history_status() {
                        updated_entry.watched_or_removed_timestamp = Some(message.timestamp);
                    }

                    send_message::status_changed_successfully(bot_data);
                    let _ = bot_data.watch_list.insert(id, updated_entry);
                // If the user is not an admin and did not add the movie himself he is not permitted to change it
                } else {
                    send_message::insufficient_permissions_error(bot_data);
                }
            },
            None => {
                send_message::movie_id_not_found_error(bot_data, &id);
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
                    // Only allow change of status if the user has the admin role
                    if user_is_admin {
                        let updated_entry = WatchListEntry {
                            movie: watch_list_entry.movie.clone(),
                            status: new_status.clone(),
                            user: watch_list_entry.user.clone(),
                            watched_or_removed_timestamp: Some(datetime),
                            ..*watch_list_entry
                        };

                        send_message::status_changed_successfully(bot_data);
                        let _ = bot_data.watch_list.insert(id, updated_entry);
                    // If the user is not an admin and did not add the movie himself he is not permitted to change it
                    } else {
                        send_message::insufficient_permissions_error(bot_data);
                    }
                },
                None => {
                    send_message::movie_id_not_found_error(bot_data, &id);
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
        let _ = send_message::movie_information(bot_data, entry, false, false);
    } else {
        send_message::movie_id_not_found_error(bot_data, &id);
    }
}

/**
 * Shows the movie information to the movie title or an error message if th etitle does not exist
 */
pub fn show_movie_by_title(bot_data: &crate::BotData, title: String) {
    if let Some(id) = get_movie_id_in_watch_list(title.as_str(), &bot_data.watch_list) {
        return show_movie_by_id(bot_data, id);
    } else {
        send_message::movie_title_not_found_error(bot_data, title);
    }
}

/** 
 * Updates the movie limit per user and sends an info message
 */
pub fn set_movie_limit(bot_data: &mut crate::BotData, new_limit: u32) {
    let message = bot_data.message.clone().expect("Passing of message to set_movie_limit failed.");
    
    if !is_user_administrator(bot_data, message.author.id) {
        return send_message::insufficient_permissions_error(bot_data);
    }

    let old_limit = bot_data.movie_limit_per_user;
    bot_data.movie_limit_per_user = new_limit;

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed
            .title("Filmlimit aktualisiert")
            .description(format!("Das Filmlimit wurde von `{}` auf `{}` geändert.", old_limit, new_limit).as_str())
            .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Sends a message showing the current movie limit
 */
pub fn show_movie_limit(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.clone().expect("Passing of message to show_movie_limit function failed.").channel_id,
        "",
        |embed| embed
            .title("Filmlimit")
            .description(format!("Das aktuelle Filmlimit beträgt `{}` pro Nutzer.", bot_data.movie_limit_per_user).as_str())
            .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Returns the watch list id of the movie if the movie was found
 */
pub fn get_movie_id_in_watch_list(title: &str, watch_list: &HashMap<u32, WatchListEntry>) -> Option<u32> {
    for (id, entry) in watch_list {
        if entry.movie.movie_title.to_lowercase() == title.to_lowercase() || entry.movie.original_title.to_lowercase() == title.to_lowercase() {
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
        if entry.movie.tmdb_id == tmdb_id {
            return Some(id);
        }
    }
    None
}

/**
 * Extracts the three earliest movies from the watch list
 */
pub fn get_three_earliest_movie_ids(bot_data: &crate::BotData) -> Vec<&u32> {
    let mut all_ids : Vec<&u32> = bot_data.watch_list.keys().collect();
    all_ids.sort();

    if all_ids.len() >= 3 {
        return all_ids[0..3].to_vec();
    } else {
        return all_ids[0..all_ids.len()].to_vec();
    }
}