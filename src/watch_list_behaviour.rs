use crate::movie_behaviour::{SortedMovieList, WatchListEntry, UserSortedMovieListVectorEntry, get_movie_link};
use std::collections::HashMap;
use itertools::Itertools;
use crate::general_behaviour::{timestamp_to_string};
use crate::{COLOR_BOT};

/**
 * Formats the watch list hash map and sends it as an embedded message
 */
pub fn show_watch_list(bot_data: &mut crate::BotData, order: String) {
    let message = bot_data.message.as_ref().expect("Passing message to show_watch_list function failed.");

    // First check if there was already a watch list waiting for reactions
    if let Some((idx, message_id)) = bot_data.wait_for_reaction.iter().enumerate()
        .find_map(|(idx, x)| if let crate::general_behaviour::WaitingForReaction::WatchListPagination(message_id, _, _) = x {
            Some((idx, message_id))
        } else {
            None
        })
    {
        // If there was, delete the reactions of the bot on that message and remove it from the
        // wait_for_reaction vector
        let _ = bot_data.bot.delete_reaction(
            message.channel_id,
            *message_id,
            None,
            discord::model::ReactionEmoji::Unicode("⬅️".to_string())
        );

        let _ = bot_data.bot.delete_reaction(
            message.channel_id,
            *message_id,
            None,
            discord::model::ReactionEmoji::Unicode("➡️".to_string())
        );

        bot_data.wait_for_reaction.remove(idx);
    }

    let mut watch_list_string: String = String::new();
    let watch_list_count = count_watch_list_movies(bot_data);
    let total_pages: usize;
    let sorted_watch_list_enum_option: Option<SortedMovieList>;

    if watch_list_count > 0 {
        // If the ordering by user is demanded
        if order == "user" {
            let user_sorted_watch_list = create_user_sorted_watch_list_vector(bot_data);
            total_pages = user_sorted_watch_list.iter().map(|x| x.number_of_pages_required).sum();
            
            watch_list_string += generate_user_sorted_watch_list_page_string(&user_sorted_watch_list, 1).as_str();

            sorted_watch_list_enum_option = Some(SortedMovieList::WatchListUserSorted(total_pages, user_sorted_watch_list));
        } 
        // If the ordering should be by id
        else {
            let id_sorted_watch_list: Vec<(u32, WatchListEntry)> = bot_data.watch_list.iter().sorted()
                .filter_map(|(id, entry)| if entry.status.is_watch_list_status() {
                    Some((*id, entry.clone()))
                } else {
                    None
                })
                .collect();
            total_pages = (id_sorted_watch_list.len() as f64 / crate::MAX_ENTRIES_PER_PAGE as f64).ceil() as usize;

            watch_list_string += generate_id_sorted_watch_list_page_string(&id_sorted_watch_list, 1).as_str();

            sorted_watch_list_enum_option = Some(SortedMovieList::WatchListIdSorted(total_pages, id_sorted_watch_list));
        }
    } else {
        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
                .title("Filmliste")
                .description("Es sind zur Zeit **0** Filme auf der Liste")
                .color(COLOR_BOT)
        );
        return;
    }

    // We know, that this can't be none now, so unwrap the value
    let sorted_watch_list_enum = sorted_watch_list_enum_option.unwrap();
    
    if let Ok(message) = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed
            .title("Filmliste")
            .description(watch_list_string.as_str())
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
            crate::general_behaviour::WaitingForReaction::WatchListPagination(
                message.id, 
                sorted_watch_list_enum,
                1
            )
        );
    }
}

/**
 * Generates the description text for the watch list sorted by user
 */
pub fn generate_user_sorted_watch_list_page_string(
    watch_list_vector: &Vec<UserSortedMovieListVectorEntry>,
    page_to_show: usize
) -> String {
    let mut watch_list_string = String::new();
    
    let watch_list_count: usize = watch_list_vector.iter().map(|x| x.entries.len()).sum();
    if watch_list_count == 1 {
        watch_list_string += format!("Es ist zur Zeit **{}** Film auf der Liste\n\n", watch_list_count).as_str();
    } else {
        watch_list_string += format!("Es sind zur Zeit **{}** Filme auf der Liste\n\n", watch_list_count).as_str();
    }

    if watch_list_count == 0 {
        return watch_list_string;
    }

    watch_list_string += format!("Die Filme werden geordnet nach dem Nutzer angezeigt, welcher sie hinzugefügt hat.\n\n").as_str();

    let mut accumulated_pages: usize = 0;
    let mut entry_to_show: Option<UserSortedMovieListVectorEntry> = None;

    // Find the entry in the vector that contains the page to show
    for entry in watch_list_vector {
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

    watch_list_string += format!("Hinzugefügt von **{}**\n\n", entry_to_show.user_name).as_str();

    // Now iterate over the entry vector
    entry_to_show.entries.iter()
        // Returns the entries with an index
        .enumerate()
        // Get only those elements that should be shown on that page
        .filter(|(idx, _)| *idx >= first_index_to_show && *idx < first_index_to_show + crate::MAX_ENTRIES_PER_PAGE)
        // For each of those append the string to the watch list
        .for_each(
            |(_, entry)| {
                watch_list_string += format!(" {} [**{}**]({})\n> `{:0>4}` | hinzugefügt am {}\n\n", 
                        entry.1.status.get_emoji(), 
                        entry.1.movie.movie_title, 
                        get_movie_link(entry.1.movie.tmdb_id, false), 
                        entry.0.to_string(), 
                        timestamp_to_string(&entry.1.added_timestamp, false)
                    )
                    .as_str()
            }
        );

    watch_list_string
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
 * Returns a watch list vector that is sorted by users and contains all information needed for paginating the entries
 */
fn create_user_sorted_watch_list_vector(bot_data: &crate::BotData) -> Vec<UserSortedMovieListVectorEntry> {
    let user_sorted_watch_list_hash_map = create_user_sorted_watch_list(bot_data);
    let get_number_pages_as_usize = |entries_len: usize| -> usize {
        (entries_len as f64 / crate::MAX_ENTRIES_PER_PAGE as f64).ceil() as usize
    };

    user_sorted_watch_list_hash_map.iter()
        .map(|(name, entries)| UserSortedMovieListVectorEntry {
            user_name: (*name).clone(),
            number_of_pages_required: get_number_pages_as_usize(entries.len()),
            entries: entries.iter().map(|(id, value)| (*id, (*value).clone())).collect(),
        })
        .collect()
}

/**
 * Generates the paginated description text for the id sorted watch list
 */
pub fn generate_id_sorted_watch_list_page_string(id_sorted_watch_list: &Vec<(u32, WatchListEntry)>, page_to_show: usize) -> String {
    let mut watch_list_string = String::new();

    let watch_list_count = id_sorted_watch_list.len();
    if watch_list_count == 1 {
        watch_list_string += format!("Es ist zur Zeit **{}** Film auf der Liste\n\n", watch_list_count).as_str();
    } else {
        watch_list_string += format!("Es sind zur Zeit **{}** Filme auf der Liste\n\n", watch_list_count).as_str();
    }

    if watch_list_count == 0 {
        return watch_list_string;
    }

    watch_list_string += format!("Die Filme werden geordnet nach ID angezeigt.\n\n").as_str();

    let first_index_to_show = (page_to_show - 1) * crate::MAX_ENTRIES_PER_PAGE;

    // For every movie entry
    id_sorted_watch_list.iter()
        // Get the index of every element
        .enumerate()
        // Get only those elements that belong on that page
        .filter(|(idx, (_, _))| if *idx >= first_index_to_show && *idx < first_index_to_show + crate::MAX_ENTRIES_PER_PAGE {
            true
        } else { 
            false
        })
        // Now build the watch_list_string for those
        .for_each(
            |(_, (id, entry))| {
                watch_list_string += format!(" {} [**{}**]({})\n> `{:0>4}` | hinzugefügt von **{}** am {}\n\n", 
                    entry.status.get_emoji(), 
                    entry.movie.movie_title, 
                    get_movie_link(entry.movie.tmdb_id, false), 
                    id.to_string(), 
                    entry.user, 
                    timestamp_to_string(&entry.added_timestamp, false)
                ).as_str();
            });

    watch_list_string
}

/**
 * Counts all movies from the list with watch list status and returns the count
 */
fn count_watch_list_movies(bot_data: &crate::BotData) -> usize {
    bot_data.watch_list.iter()
        .filter(|(_, entry)| entry.status.is_watch_list_status())
        .count()
}