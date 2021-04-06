use discord::{model as Model, Discord};
use std::collections::HashMap;
use chrono::DateTime;

pub struct WatchListEntry {
    id: u32,
    user: String,
    timestamp: DateTime<chrono::FixedOffset>,
}

/**
 * Adds a movie to the watch list hash map
 */
pub fn add_movie(bot_data: &mut crate::BotData, title: &str) {
    println!("Adding movie with title '{}'", title);

    let message = bot_data.message.as_ref().expect("Passing message to add_movie function failed");

    // The movie title is already in the watch list
    if bot_data.watch_list.contains_key(&title.to_string()) {
        let previous_entry = bot_data.watch_list.get(&title.to_string()).expect("Accessing the watch list has failed inside the add_movie function.");

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
            id: bot_data.next_movie_id,
            user: message.author.name.clone(),
            timestamp: message.timestamp,
        };

        bot_data.next_movie_id += 1;

        bot_data.watch_list.insert(title.to_string(), new_entry);

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
    println!("Removing movie {}", title);

    let message = bot_data.message.as_ref().expect("Passing message to remove_movie_by_title function failed.");

    let user_is_admin: bool = is_user_administrator(bot_data, message.author.id);

    let movie = bot_data.watch_list.get(&title.to_string());
    match movie {
        Some(watch_list_entry) => {
            if watch_list_entry.user == message.author.name {
                let _ = bot_data.watch_list.remove(&title.to_string());
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Removed movie '{}'", title).as_str(),
                    "",
                    false
                );
            } else if user_is_admin {
                let watch_list_entry_user = watch_list_entry.user.clone();
                let _ = bot_data.watch_list.remove(&title.to_string());
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Removed movie '{}' added by user {}", title, watch_list_entry_user).as_str(),
                    "",
                    false
                );
            } else {
                let _ = bot_data.bot.send_message(
                    message.channel_id,
                    format!("Insufficient permissions to remove the movie '{}' added by user {}.", title, watch_list_entry.user).as_str(),
                    "",
                    false
                );
            }
            
        },
        None => {
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
    let mut associated_title: String = "".to_string();
    for (title, movie_entry) in &bot_data.watch_list {
        if movie_entry.id == id {
            associated_title = title.clone();
            break;
        }
    }
    
    remove_movie_by_title(bot_data, &associated_title.as_str());
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