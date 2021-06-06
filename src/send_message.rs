use crate::{COLOR_ERROR, COLOR_SUCCESS, COLOR_WARNING, movie_behaviour, general_behaviour};

/**
 * Sends a message that the user has insufficient permissions
 */
pub fn insufficient_permissions_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.clone().expect("Passing message to send_insufficient_permissions_error_message failed.").channel_id,
        "",
        |embed| embed
            .title("Keine Berechtigung")
            .description("Leider besitzt du nicht die benötigte Berechtigung um das zu tun.")
            .color(crate::COLOR_ERROR)
    );
}

/**
 * Sends an embedded message that the movie was already added by someone
 */
pub fn movie_already_exists(bot_data: &crate::BotData, id: u32, tmdb_id: u64) {

    let message = bot_data.message.as_ref().expect("Passing message to send_movie_already_exists_message function failed");
    let previous_entry = bot_data.watch_list.get(&id).expect("Accessing the watch list has failed inside the send_movie_already_exists_message function.");

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed
        .title(format!("{}", previous_entry.movie.movie_title).as_str())
        .url(movie_behaviour::get_movie_link(tmdb_id, false).as_str())
        .thumbnail(movie_behaviour::generate_poster_link(&previous_entry.movie.poster_path).as_str())
        .description(
            format!("**{}** hat diesen Film bereits am *{}* hinzugefügt.\nFalls du einen anderen Film meinst versuche das Hinzufügen durch einen IMDb Link.", 
                previous_entry.user,
                general_behaviour::timestamp_to_string(&previous_entry.added_timestamp, true),
            )
            .as_str()
        )
        .color(COLOR_ERROR)
    );
}

/**
 * Sends an error message, that the user already has too many movies in the watch list
 */
pub fn user_has_too_many_movies_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.clone().expect("Passing message to send_user_has_too_many_movies_error_message failed.").channel_id,
        "",
        |embed| embed
            .title("Zu viele Filme hinzugefügt")
            .description(
                format!("Leider hast du bereits zu viele Filme zur Liste hinzugefügt. 
                Das aktuelle Limit beträgt `{}` pro Nutzer.", 
                    bot_data.movie_limit_per_user
                )
                .as_str()
            )
            .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Takes a movie entry and sends an embedded message with all information of the movie
 */
pub fn movie_information(bot_data: &crate::BotData, movie_entry: &movie_behaviour::WatchListEntry, new_movie: bool, ask_confirmation: bool) -> Result<discord::model::Message, discord::Error> {
    let message = bot_data.message.as_ref().expect("Passing message to send_movie_information_message function failed.");

    if new_movie {
        bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
            .title(format!("{}", movie_entry.movie.movie_title).as_str())
            .url(
                movie_behaviour::get_movie_link(movie_entry.movie.tmdb_id, false).as_str()
            )
            .description(
                movie_entry.movie.overview.clone().as_str()
            )
            .image(movie_behaviour::generate_poster_link(&movie_entry.movie.poster_path).as_str())
            .color(COLOR_SUCCESS)
            .fields(|fields| fields
                .field("Originaltitel", movie_entry.movie.original_title.as_str(), true)
                .field("Originalsprache", movie_entry.movie.original_language.as_str(), true)
                .field("Erschienen", general_behaviour::timestamp_to_string(&movie_entry.movie.release_date, false).as_str(), true)
                .field("Genres", movie_entry.movie.genres.as_str(), true)
                .field("Dauer", format!("{} min", movie_entry.movie.runtime).as_str(), true)
                .field("Budget", movie_entry.movie.budget.as_str(), true)
            )
            .footer(|footer| footer
                .text(format!("{}", if ask_confirmation {"Meintest du diesen Film?"} else {""}).as_str())
            )
            .thumbnail(general_behaviour::get_tmdb_attribution_icon_url())
        )
    } else {
        bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed
            .title(format!("{}", movie_entry.movie.movie_title).as_str())
            .url(
                movie_behaviour::get_movie_link(movie_entry.movie.tmdb_id, false).as_str()
            )
            .description(
                movie_entry.movie.overview.clone().as_str()
            )
            .image(movie_behaviour::generate_poster_link(&movie_entry.movie.poster_path).as_str())
            .color(COLOR_SUCCESS)
            .fields(|fields| fields
                .field("Originaltitel", movie_entry.movie.original_title.as_str(), true)
                .field("Originalsprache", movie_entry.movie.original_language.as_str(), true)
                .field("Erschienen", general_behaviour::timestamp_to_string(&movie_entry.movie.release_date, false).as_str(), true)
                .field("Genres", movie_entry.movie.genres.as_str(), true)
                .field("Dauer", format!("{} min", movie_entry.movie.runtime).as_str(), true)
                .field("Budget", movie_entry.movie.budget.as_str(), true)
                .field("Hinzugefügt von", format!("<@{}>", movie_entry.user_id).as_str(), true)
                .field("Hinzugefügt am", general_behaviour::timestamp_to_string(&movie_entry.added_timestamp, true).as_str(), true)
                .field("Status", format!("{}", movie_entry.status.get_emoji()).as_str(), true)
            )
            .thumbnail(general_behaviour::get_tmdb_attribution_icon_url())
        )
    }
}

/**
 * Sends an error message that the movie could not be found
 */
pub fn movie_id_not_found_error(bot_data: &crate::BotData, id: &u32) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.clone().expect("Passing message to send_message::movie_not_found_error failed.").channel_id,
        "",
        |embed| embed
            .title("Film nicht gefunden")
            .description(
                format!("Ein Film mit der ID `{:0>4}` konnte weder in der Filmliste noch im Verlauf gefunden werden.", id).as_str(),
            )
            .color(COLOR_ERROR)
    );
}

/**
 * Sends an error message, stating, that the movie with the title could not be found
 */
pub fn movie_title_not_found_error(bot_data: &crate::BotData, title: String) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing of message to send_message::movie_title_not_found_error function failed.").channel_id,
        "",
        |embed| embed
        .title("Film nicht gefunden")
        .description(
            format!("Ein Film mit dem Namen *{}* konnte weder in der Filmliste noch im Verlauf gefunden werden.", title).as_str()
        )
        .color(COLOR_ERROR)
    );
}

/**
 * Sends a message that the status change was successful.
 */
pub fn status_changed_successfully(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.clone().expect("Passing message to send_message::status_changed_successfully failed.").channel_id,
        "",
        |embed| embed
        .title("Status geändert")
        .description("Der Status des Films wurde erfolgreich geändert.")
        .color(COLOR_SUCCESS)
    );
}

/**
 * Informs the user, that the movie was removed successfully
 */
pub fn movie_removed_successfully(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.clone().expect("Passing message to send_message::movie_removed_successfully failed.").channel_id,
        "",
        |embed| embed
            .title("Film entfernt.")
            .description("Der Film wurde erfolgreich entfernt.")
            .color(COLOR_WARNING)
    );
}

/**
 * Tells the user that he already own a vote
 */
pub fn user_already_owns_a_vote_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::user_already_owns_a_vote_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Du besitzt bereits eine Abstimmung.")
        .description("Bitte beende zunächst die Abstimmung bevor du eine neue eröffnest.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Informs the user, that he tried to create a vote with too many options, which is the reason why it couldn't be created
 */
pub fn not_enough_emojis_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::not_enough_emojis_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Zu viele Vote-Optionen")
        .description("Die Abstimmung konnte nicht erstellt werden, da leider zu viele Optionen hinzugefügt wurden. Versuche es bitte mit weniger Optionen erneut.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Informs the user, that the given movie (either by id or by title) could not be found in the watch_list
 */
pub fn movie_not_found_in_watchlist_error(bot_data: &crate::BotData, option_string: String) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::movie_not_found_in_watchlist_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Film konnte nicht gefunden werden")
        .description(
            format!("Die Abstimmung konnte nicht erstellt werden, da die Option '{}' nicht in der Filmliste gefunden werden konnte.", option_string).as_str()
        )
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Informs the user, that the given id parameter had the wrong format
 */
pub fn wrong_vote_parameter_error(bot_data: &crate::BotData, parameter: String) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::wrong_vote_parameter_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Abstimmungsparameter hat falsches Format")
        .description(
            format!("Die Abstimmung konnte nicht erstellt werden, da die Option '{}' das falsche Format für eine ID hat. Bitte verwende nur Zahlen.", parameter).as_str()
        )
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Sends an error message that tells the user, that the vote message could not be sent to the channel
 */
pub fn vote_message_failed_to_send_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::vote_message_failed_to_send_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Senden fehlgeschlagen")
        .description("Aus unerklärlichen Gründen ist das Senden der Abstimmungsnachricht leider fehlgeschlagen.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Sends an error message that the user has no vote yet
 */
pub fn user_has_no_vote_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::user_has_no_vote_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Keine Abstimmung")
        .description("Es sieht so aus als ob du aktuell keine Abstimmung besitzt.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Sends the error message that the vote could not be found in the list of votes
 */
pub fn vote_not_found_error(bot_data: &mut crate::BotData, user_id: &discord::model::UserId) {
    if let Ok(private_channel) = bot_data.bot.create_private_channel(*user_id) {
        let _ = bot_data.bot.send_embed(
            private_channel.id, 
            "",
            |embed| embed
            .title("Abstimmung existiert nicht")
            .description("Vielleicht hast du versucht auf eine alte Abstimmung zu reagieren, oder der Nutzer hat die Abstimmung erneut in den Kanal gesendet?")
            .color(crate::COLOR_ERROR)
        );
    }
}

/**
 * Sends the error message, that the given reaction emoji to a given vote is not
 * part of that vote, meaning the user has reacted with an additional emoji.
 * The user should be informed, that the emoji does not change anything in the vote
 * and that the user should react with an emoji that is part of the vote
 */
pub fn emoji_not_part_of_vote_info(bot_data: &mut crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::emoji_not_part_of_vote_info failed.").channel_id, 
        "",
        |embed| embed
        .title("Emoji ist nicht Teil der Abstimmung")
        .description("Danke für deine Reaktion auf meine Nachricht, aber ich bin verpflichtet dir mitzuteilen, dass dieses Emoji nicht Teil der Abstimmung ist. Falls du eine Stimme abgeben möchtest reagiere bitte mit einem passenden Emoji.")
        .color(crate::COLOR_INFORMATION)
    );
}