use discord::{model as Model, Discord};
use chrono::DateTime;
use std::collections::HashMap;

pub struct WatchListEntry {
    movie_title: String,
    user: String,
    timestamp: DateTime<chrono::FixedOffset>,
}

/**
 * Adds a movie to the watch list hash map
 */
pub fn add_movie(bot_data: &mut crate::BotData, title: &str) {
    println!("Adding movie with title '{}'", title);

    let message = bot_data.message.as_ref().expect("Passing message to add_movie function failed");

    if let Some(id) = get_movie_id_in_watch_list(title, &bot_data.watch_list) {
        let previous_entry = bot_data.watch_list.get(&id).expect("Accessing the watch list has failed inside the add_movie function.");

        let _ = bot_data.bot.send_message(
            message.channel_id,
            format!("The user '{}' has already added this movie on {}", 
                previous_entry.user,
                previous_entry.timestamp.format("%A, %d.%m.%Y"),
            )
            .as_str(),
            "",
            false
        );
    } else {
        let new_entry = WatchListEntry {
            movie_title: title.to_string(),
            user: message.author.name.clone(),
            timestamp: message.timestamp,
        };

        bot_data.watch_list.insert(bot_data.next_movie_id, new_entry);
        
        bot_data.next_movie_id += 1;

        let _ = bot_data.bot.send_message(
            message.channel_id,
            format!("Added movie '{}' to the watch list.", title).as_str(),
            "",
            false
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
            let _ = bot_data.bot.send_message(
                message.channel_id,
                format!("A movie with the title '{}' was not found.", title).as_str(),
                "",
                false
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
            let watch_list_entry_title = watch_list_entry.movie_title.clone();

            if watch_list_entry.user == message.author.name {
                let _ = bot_data.watch_list.remove(&id);
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Removed movie '{}'", watch_list_entry_title).as_str(),
                    "",
                    false
                );
            } else if user_is_admin {
                let watch_list_entry_user = watch_list_entry.user.clone();
                let _ = bot_data.watch_list.remove(&id);
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Removed movie '{}' added by user {}", watch_list_entry_title, watch_list_entry_user).as_str(),
                    "",
                    false
                );
            } else {
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Insufficient permissions to remove the movie '{}' added by user {}.", watch_list_entry_title, watch_list_entry.user).as_str(),
                    "",
                    false
                );
            }
            
        },
        None => {
            let _ = bot_data.bot.send_message(
                message.channel_id,
                format!("A movie with the id '{}' was not found.", id).as_str(),
                "",
                false
            );
        }
    }
}

/**
 * Edits the title of a movie list entry via the id
 */
pub fn edit_movie_by_id(bot_data: &mut crate::BotData, id: u32, new_title: &str) {
    let message = bot_data.message.as_ref().expect("Passing message to remove_movie_by_title function failed.");

    let user_is_admin: bool = is_user_administrator(bot_data, message.author.id);

    let movie = bot_data.watch_list.get(&id);
    match movie {
        Some(watch_list_entry) => {
            let watch_list_entry_title = watch_list_entry.movie_title.clone();

            if watch_list_entry.user == message.author.name {
                let updated_entry = WatchListEntry {
                    movie_title: new_title.to_string(),
                    user: watch_list_entry.user.clone(),
                    ..*watch_list_entry
                };
                let _ = bot_data.watch_list.insert(id, updated_entry);
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Changed movie '{}' to '{}'", watch_list_entry_title, new_title).as_str(),
                    "",
                    false
                );
            } else if user_is_admin {
                let watch_list_entry_user = watch_list_entry.user.clone();

                let updated_entry = WatchListEntry {
                    movie_title: new_title.to_string(),
                    user: watch_list_entry.user.clone(),
                    ..*watch_list_entry
                };
                let _ = bot_data.watch_list.insert(id, updated_entry);
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Changed movie '{}' added by user {} to '{}'", watch_list_entry_title, watch_list_entry_user, new_title).as_str(),
                    "",
                    false
                );
            } else {
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Insufficient permissions to edit the movie '{}' added by user {}.", watch_list_entry_title, watch_list_entry.user).as_str(),
                    "",
                    false
                );
            }
            
        },
        None => {
            let _ = bot_data.bot.send_message(
                message.channel_id,
                format!("A movie with the id {} was not found.", id).as_str(),
                "",
                false
            );
        }
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