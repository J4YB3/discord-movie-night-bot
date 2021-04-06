use discord::{model as Model, Discord};
use std::collections::HashMap;
use chrono::DateTime;

pub struct WatchListEntry {
    user: String,
    timestamp: DateTime<chrono::FixedOffset>,
}

/**
 * Adds a movie to the watch list hash map
 */
pub fn add_movie(bot_data: &mut crate::BotData, title: &str) {
    println!("Adding movie with title '{}'", title);

    let message = bot_data.message.as_ref().expect("Passing of message to add_movie function failed");

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
            user: message.author.name.clone(),
            timestamp: message.timestamp,
        };

        bot_data.watch_list.insert(title.to_string(), new_entry);
    }
}