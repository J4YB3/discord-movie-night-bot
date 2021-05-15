use discord::{model as Model};
use chrono::DateTime;
use std::collections::HashMap;
use itertools::Itertools;
use std::{fmt, cmp::Ordering, str::FromStr};
use crate::{COLOR_ERROR, COLOR_SUCCESS, COLOR_WARNING, COLOR_BOT};
use crate::general_behaviour::timestamp_to_string;

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

/**
 * Adds a movie to the watch list hash map
 */
pub fn add_movie(bot_data: &mut crate::BotData, title: &str) {
    let message = bot_data.message.as_ref().expect("Passing message to add_movie function failed");

    if let Some(id) = get_movie_id_in_watch_list(title, &bot_data.watch_list) {
        let previous_entry = bot_data.watch_list.get(&id).expect("Accessing the watch list has failed inside the add_movie function.");

        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
            .description(
                format!("**{}** hat diesen Film bereits am *{}* hinzugefügt.", 
                    previous_entry.user,
                    timestamp_to_string(&previous_entry.added_timestamp),
                )
                .as_str()
            )
            .color(COLOR_ERROR)
        );
    } else {
        let new_entry = WatchListEntry {
            movie_title: title.to_string(),
            user: message.author.name.clone(),
            added_timestamp: message.timestamp,
            watched_or_removed_timestamp: None,
            status: MovieStatus::NotWatched,
            user_id: message.author.id,
        };

        bot_data.watch_list.insert(bot_data.next_movie_id, new_entry);
        
        bot_data.next_movie_id += 1;

        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed.description(
                format!("Der Film wurde zur Liste hinzugefügt.\n`{:0>4}` {} **{}**", 
                    bot_data.next_movie_id - 1, 
                    MovieStatus::NotWatched.get_emoji(),
                    title
                ).as_str()
            )
            .color(COLOR_SUCCESS)
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
                            timestamp_to_string(&watch_list_entry.added_timestamp)
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
                            timestamp_to_string(&watch_list_entry.added_timestamp),
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
                            timestamp_to_string(&watch_list_entry.added_timestamp),
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
 * Edits the title of a movie list entry via the id
 */
pub fn edit_movie_by_id(bot_data: &mut crate::BotData, id: u32, new_title: &str) {
    let message = bot_data.message.as_ref().expect("Passing message to edit_movie_by_id function failed.");

    let user_is_admin: bool = is_user_administrator(bot_data, message.author.id);

    let movie = bot_data.watch_list.get(&id);
    match movie {
        Some(watch_list_entry) => {
            let watch_list_entry_title = watch_list_entry.movie_title.clone();

            // If the user himself added the movie
            if watch_list_entry.user == message.author.name {
                let updated_entry = WatchListEntry {
                    movie_title: new_title.to_string(),
                    user: watch_list_entry.user.clone(),
                    status: watch_list_entry.status.clone(),
                    ..*watch_list_entry
                };
                let _ = bot_data.bot.send_embed(
                    message.channel_id,
                    "",
                    |embed| embed.title("Filmeintrag geändert")
                    .description(
                        format!("
                        **von**
                        `{:0>4}` {} **{}** | hinzugefügt am *{}*\n
                        **zu**
                        `{:0>4}` {} **{}** | hinzugefügt am *{}*", 
                            id, 
                            watch_list_entry.status.get_emoji(), 
                            watch_list_entry_title,
                            timestamp_to_string(&watch_list_entry.added_timestamp),
                            id, 
                            updated_entry.status.get_emoji(), 
                            updated_entry.movie_title,
                            timestamp_to_string(&updated_entry.added_timestamp),
                        ).as_str()
                    )
                    .color(COLOR_SUCCESS)
                );
                let _ = bot_data.watch_list.insert(id, updated_entry);
            // if the user did not add the movie himself but has the admin role, he is permitted to change movies by others
            } else if user_is_admin {
                let updated_entry = WatchListEntry {
                    movie_title: new_title.to_string(),
                    user: watch_list_entry.user.clone(),
                    status: watch_list_entry.status.clone(),
                    ..*watch_list_entry
                };
                let _ = bot_data.bot.send_embed(
                    message.channel_id,
                    "",
                    |embed| embed.title(format!("Film hinzugefügt von <@{}> wurde geändert.", updated_entry.user_id).as_str())
                    .description(
                        format!("
                        **von**
                        `{:0>4}` {} **{}** | hinzugefügt am *{}*\n
                        **zu**
                        `{:0>4}` {} **{}** | hinzugefügt am *{}*", 
                            id, 
                            watch_list_entry.status.get_emoji(), 
                            watch_list_entry.movie_title,
                            timestamp_to_string(&watch_list_entry.added_timestamp),
                            id, 
                            updated_entry.status.get_emoji(), 
                            updated_entry.movie_title,
                            timestamp_to_string(&updated_entry.added_timestamp)
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
                        format!("Du hat nicht genügend Rechte um den Film zu ändern. Klagen bitte an das Verfassungsgericht.
                        `{:0>4}` {} **{}** | hinzugefügt am *{}* von <@{}>", 
                            id, 
                            watch_list_entry.status.get_emoji(), 
                            watch_list_entry.movie_title,
                            timestamp_to_string(&watch_list_entry.added_timestamp),
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
                    watch_list_string += format!(" {} **{}** | hinzugefügt am *{}*\n", entry.status.get_emoji(), entry.movie_title, timestamp_to_string(&entry.added_timestamp)).as_str();
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
                    watch_list_string += format!(" {} **{}** | hinzugefügt von **{}** am *{}*\n", entry.status.get_emoji(), entry.movie_title, entry.user, timestamp_to_string(&entry.added_timestamp)).as_str();
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
                            )
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
                        timestamp_to_string(&entry.added_timestamp)).as_str();
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
                        user: watch_list_entry.user.clone(),
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
                                timestamp_to_string(&watch_list_entry.added_timestamp),
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
 * Checks all roles of the user for admin permissions and returns true if the user has at least one
 * role with those permissions
 */
fn is_user_administrator(bot_data: &crate::BotData, user_id: Model::UserId) -> bool {
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
        if entry.movie_title == title {
            return Some(*id);
        }
    }
    None
}