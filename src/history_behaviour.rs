use crate::movie_behaviour::{SortedMovieList, WatchListEntry, UserSortedMovieListVectorEntry, get_movie_link, MovieStatus};
use std::collections::HashMap;
use itertools::Itertools;
use crate::general_behaviour::{timestamp_to_string};
use crate::{COLOR_BOT};

/**
 * Formats the watch list hash map as a movie history and sends it as an embedded message
 */
pub fn show_history(bot_data: &mut crate::BotData, order: String, reverse: Bool) {
    let message = bot_data.message.as_ref().expect("Passing message to show_history function failed.");

    // First check if there was already a history message waiting for reactions
    if let Some((idx, message)) = bot_data.wait_for_reaction.iter().enumerate()
        .find_map(|(idx, x)| if let crate::general_behaviour::WaitingForReaction::HistoryPagination(message, _, _) = x {
            Some((idx, message))
        } else {
            None
        })
    {
        // If there was, delete the reactions of the bot on that message and remove it from the
        // wait_for_reaction vector
        crate::general_behaviour::remove_reactions_on_message(bot_data, message, vec!["⬅️", "➡️"]);

        bot_data.wait_for_reaction.remove(idx);
    }

    let mut history_string = String::new();
    let history_count = count_history_movies(bot_data);
    let sorted_movie_list_enum_option: Option<SortedMovieList>;
    let total_pages: usize;

    if history_count > 0 {
        // If the ordering by user is demanded
        if order == "user" {
            let user_sorted_history = create_user_sorted_history_vector(bot_data);
            total_pages = user_sorted_history.iter().map(|x| x.number_of_pages_required).sum();

            history_string += generate_user_sorted_history_page_string(
                    &user_sorted_history, 
                    1
                ).as_str();
            
            sorted_movie_list_enum_option = Some(SortedMovieList::HistoryUserSorted(total_pages, user_sorted_history));
        } 
        // If the ordering should be by date
        else {
            // Get all movies from the history
            let mut date_sorted_history: Vec<(u32, WatchListEntry)> = bot_data.watch_list.iter()
                .filter_map(|(id, entry)| if entry.status.is_history_status() {
                    Some((*id, entry.clone()))
                } else {
                    None
                })
                .collect();
            
            // And sort them by date
            date_sorted_history.sort_by(
                |a, b| 
                    a.1.watched_or_removed_timestamp
                        .expect("Movie did not have a watched_or_removed timestamp in show_history")
                        .cmp(
                            &b.1.watched_or_removed_timestamp
                            .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                        )
            );
            if eq(reverse,true) {
                date_sorted_history = date_sorted_history.reverse()
            }
            total_pages = (date_sorted_history.len() as f64 / crate::MAX_ENTRIES_PER_PAGE as f64).ceil() as usize;

            history_string += generate_date_sorted_history_page_string(&date_sorted_history, 1).as_str();
            sorted_movie_list_enum_option = Some(SortedMovieList::HistoryDateSorted(total_pages, date_sorted_history));
        }
    } else {
        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
                .title("Verlauf")
                .description("Es sind zur Zeit **0** Filme im Verlauf")
                .color(COLOR_BOT)
        );
        return;
    }

    // We know, that this can't be none now, so unwrap the value
    let sorted_movie_list_enum = sorted_movie_list_enum_option.unwrap();
    
    if let Ok(message) = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed
            .title("Verlauf")
            .description(history_string.as_str())
            .color(COLOR_BOT)
            .footer(|footer| footer.text(format!("Seite {}/{}", 1, total_pages).as_str()))
    ) {
        let _ = bot_data.bot.add_reaction(
            message.channel_id,
            message.id,
            discord::model::ReactionEmoji::Unicode("⬅️".to_string())
        );

        let _ = bot_data.bot.add_reaction(
            message.channel_id,
            message.id,
            discord::model::ReactionEmoji::Unicode("➡️".to_string())
        );

        bot_data.wait_for_reaction.push(
            crate::general_behaviour::WaitingForReaction::HistoryPagination(
                message, 
                sorted_movie_list_enum,
                1
            )
        );
    }
}

/**
 * Generates the paginated description for the user sorted history and for the given page
 */
pub fn generate_user_sorted_history_page_string(
    user_sorted_history: &Vec<UserSortedMovieListVectorEntry>,
    page_to_show: usize
) -> String {
    let mut history_string = String::new();
    
    let history_count: usize = user_sorted_history.iter().map(|x| x.entries.len()).sum();
    if history_count == 1 {
        history_string += format!("Es ist zur Zeit **{}** Film im Verlauf\n\n", history_count).as_str();
    } else {
        history_string += format!("Es sind zur Zeit **{}** Filme im Verlauf\n\n", history_count).as_str();
    }

    if history_count == 0 {
        return history_string;
    }

    history_string += format!("Die Filme werden geordnet nach dem Nutzer angezeigt, welcher sie hinzugefügt hat.\n\n").as_str();

    let mut accumulated_pages: usize = 0;
    let mut entry_to_show: Option<UserSortedMovieListVectorEntry> = None;

    // Find the entry in the vector that contains the page to show
    for entry in user_sorted_history {
        accumulated_pages += entry.number_of_pages_required;

        if page_to_show <= accumulated_pages {
            entry_to_show = Some(entry.clone());
            break;
        }
    }

    if entry_to_show.is_none() {
        return format!("Die Seite mit der Nummer {} gibt es in dieser Liste nicht.", accumulated_pages);
    }

    // We now know that entry_to_show is a some value, so unwrap it
    let entry_to_show = entry_to_show.unwrap();

    // Calculate the first index inside the entry vector that will be displayed on the page
    let first_index_to_show = ((page_to_show - 1) - (accumulated_pages - entry_to_show.number_of_pages_required)) * crate::MAX_ENTRIES_PER_PAGE;

    history_string += format!("Hinzugefügt von **{}**\n\n", entry_to_show.user_name).as_str();

    // Now iterate over the entry vector
    entry_to_show.entries.iter()
        // Returns the entries with an index
        .enumerate()
        // Get only those elements that should be shown on that page
        .filter(|(idx, _)| *idx >= first_index_to_show && *idx < first_index_to_show + crate::MAX_ENTRIES_PER_PAGE)
        // For each of those append the string to the watch list
        .for_each(
            |(_, entry)| {
                history_string += format!(" {} [**{}**]({})\n> `{:0>4}` | {} am {}\n\n", 
                    entry.1.status.get_emoji(), 
                    entry.1.movie.movie_title, 
                    get_movie_link(entry.1.movie.tmdb_id, false),
                    entry.0.to_string(),
                    if entry.1.status == MovieStatus::Watched {"geschaut"} else {"entfernt"},
                    timestamp_to_string(
                        &entry.1.watched_or_removed_timestamp
                            .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                        , false)
                    ).as_str();
            }
        );

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

/**
 * Returns a watch list vector that is sorted by users and contains all information needed for paginating the entries
 */
fn create_user_sorted_history_vector(bot_data: &crate::BotData) -> Vec<UserSortedMovieListVectorEntry> {
    let mut user_sorted_history_hash_map = create_user_sorted_history(bot_data);
    let get_number_pages_as_usize = |entries_len: usize| -> usize {
        (entries_len as f64 / crate::MAX_ENTRIES_PER_PAGE as f64).ceil() as usize
    };

    user_sorted_history_hash_map.iter_mut()
        .sorted()
        .map(
            |(name, entries)| {
                // Sort the entries of each user by the date they have been watched (or removed)
                entries.sort_by(
                    |a, b| 
                        a.1.watched_or_removed_timestamp
                            .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                            .cmp(
                                &b.1.watched_or_removed_timestamp
                                .expect("Movie did not have a watched_or_removed_timestamp in show_history")
                            )
                );

                UserSortedMovieListVectorEntry {
                    user_name: (*name).clone(),
                    number_of_pages_required: get_number_pages_as_usize(entries.len()),
                    entries: entries.iter().map(|(id, value)| (*id, (*value).clone())).collect(),
                }
            }
        )
        .collect()
}

/** 
 * Generates the string to the given page for the paginated history
 */
pub fn generate_date_sorted_history_page_string(date_sorted_history: &Vec<(u32, WatchListEntry)>, page_to_show: usize) -> String {
    let mut history_string = String::new();

    let history_count = date_sorted_history.len();
    if history_count == 1 {
        history_string += format!("Es ist zur Zeit **{}** Film im Verlauf\n\n", history_count).as_str();
    } else {
        history_string += format!("Es sind zur Zeit **{}** Filme im Verlauf\n\n", history_count).as_str();
    }

    if history_count == 0 {
        return history_string;
    }

    history_string += format!("Die Filme werden geordnet nach Datum angezeigt.\n\n").as_str();

    let first_index_to_show = (page_to_show - 1) * crate::MAX_ENTRIES_PER_PAGE;

    // For every movie entry
    date_sorted_history.iter()
        // Get the index of every element
        .enumerate()
        // Get only those elements that belong on that page
        .filter(|(idx, (_, _))| if *idx >= first_index_to_show && *idx < first_index_to_show + crate::MAX_ENTRIES_PER_PAGE {
            true
        } else { 
            false
        })
        // Now build the history_string for those
        .for_each(
            |(_, (id, entry))| {
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
        );

    history_string
}

/**
 * Counts all movies from the list with history status and returns the count
 */
fn count_history_movies(bot_data: &crate::BotData) -> usize {
    bot_data.watch_list.iter()
        .filter(|(_, entry)| entry.status.is_history_status())
        .count()
}