use crate::{COLOR_ERROR, COLOR_SUCCESS, COLOR_WARNING, COLOR_INFORMATION, movie_behaviour, general_behaviour};

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
        .color(COLOR_INFORMATION)
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
pub fn movie_information(bot_data: &crate::BotData, movie_entry: &movie_behaviour::WatchListEntry, new_movie: bool, ask_confirmation: bool, ask_set_watched: bool) -> Result<discord::model::Message, discord::Error> {
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
                .field("Watchlink", movie_behaviour::get_movie_link(movie_entry.movie.tmdb_id, true).as_str(), false)
                .field("Watchlist-ID", movie_behaviour::get_movie_id_in_watch_list(&movie_entry.movie.movie_title, &bot_data.watch_list), true)
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
                .field("Watchlink", movie_behaviour::get_movie_link(movie_entry.movie.tmdb_id, true).as_str(), false)
            )
            .footer(|footer| footer.text(
                    format!("{}", 
                        if ask_set_watched {
                            "Soll der Film direkt als 'Watched' Status gesetzt werden?"
                        } else {
                            ""
                        }
                    )
                    .as_str()
                )
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
pub fn movie_not_found_in_watchlist_error(bot_data: &crate::BotData, movie_title: String) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::movie_not_found_in_watchlist_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Film konnte nicht gefunden werden")
        .description(
            format!("Der Film '{}' konnte nicht in der Filmliste gefunden werden.", movie_title).as_str()
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
 * Sends an error message that the user has no vote yet
 */
pub fn other_user_has_no_vote_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::user_has_no_vote_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Keine Abstimmung")
        .description("Es sieht so aus als ob der angegebene Nutzer aktuell keine Abstimmung besitzt.")
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

/**
 * Sends an information message, that the given emoji reaction was not
 * part of the expected reactions for the message
 */
pub fn emoji_not_recognized_as_reaction_info(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::emoji_not_recognized_as_reaction_info failed.").channel_id, 
        "",
        |embed| embed
        .title("Emoji nicht erwartet")
        .description("Dieses Emoji wurde nicht als Teil der erwarteten Emojis erkannt. Bitte reagiere nur mit den vorgegebenen Emojis auf meine Nachrichten.")
        .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Sends an error message, that an unknown error occured
 */
pub fn unknown_error_occured(bot_data: &crate::BotData, err_code: u32) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::unknown_error_occured failed.").channel_id, 
        "",
        |embed| embed
        .title(format!("Unerwarteter Fehler {}", err_code).as_str())
        .description(format!("Es ist ein unerwarteter Fehler aufgetreten (Fehlercode {}). 
            Bitte kontaktiere den Programmierer", err_code).as_str())
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Sends an error message, that there is no random_movie_vote at the time
 */
pub fn no_random_movie_vote_exists_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::no_random_movie_vote_exists failed.").channel_id, 
        "",
        |embed| embed
        .title("Es existiert aktuell keine Filmabstimmung")
        .description("So wie es aussieht, gibt es aktuell keine Abstimmung über den nächsten Film. Du kannst aber gerne eine neue erstellen.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Sends an information message, that there is already a random movie vote
 */
pub fn there_is_already_a_random_movie_vote_information(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::there_is_already_a_random_movie_vote_information failed.").channel_id, 
        "",
        |embed| embed
        .title("Bestehende Filmabstimmung")
        .description("Es gibt bereits eine bestehende Filmabstimmung. Hier ist sie.")
        .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Sends the error message, that in the random movie vote there was no vote option to evaluate the results from
 */
pub fn no_movie_vote_options_in_movie_vote_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::no_movie_vote_options_in_movie_vote_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Keine Abstimmungsoptionen")
        .description("Beim Versuch die Filmabstimmung auszuwerten wurde festgestellt, dass die Abstimmung keine Optionen enthielt. Das Kommando wird daher nicht zu Ende ausgeführt, und es wird kein Watch-Link generiert.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Sens the error message, that during evaluation of the random_movie_vote the movie information message could not be sent
 */
pub fn sending_of_movie_information_message_failed_error(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::sending_of_movie_information_message_failed_error failed.").channel_id, 
        "",
        |embed| embed
        .title("Senden fehlgeschlagen")
        .description("Beim Versuch die Filmabstimmung auszuwerten konnte die Nachricht mit den Details über den Film nicht gesendet werden. Ich erstelle trotzdem einen Watch-Link für euch.")
        .color(crate::COLOR_ERROR)
    );
}

// TODO: Decide if function is needed
/**
 * Sends the watch link as embedded message for the given tmdb_id and movie_title
 */
// pub fn watch_link(bot_data: &mut crate::BotData, movie: &crate::movie_behaviour::Movie, ask_add_movie_to_watched: bool) {
//     use crate::movie_behaviour::get_movie_link;

//     let movie_watch_link = get_movie_link(movie.tmdb_id, true);

//     if let Ok(message) = bot_data.bot.send_embed(
//         bot_data.message.as_ref().expect("Passing message to send_message::sending_of_movie_information_message_failed_error failed.").channel_id, 
//         "",
//         |embed| embed
//         .title("Watch-Link")
//         .description(format!("Hier ist der Watch-Link für den Film *{}*
        
//             {}", movie.movie_title, movie_watch_link)
//             .as_str()
//         )
//         .footer(|footer| footer
//             .text(
//                 format!("{}", 
//                     if ask_add_movie_to_watched {
//                         "Soll der Film direkt als 'Watched' Status gesetzt werden?"
//                     } else {
//                         ""
//                     }
//                 )
//                 .as_str()
//             )
//         )
//         .color(crate::COLOR_INFORMATION)
//     ) {
//         if ask_add_movie_to_watched {
//             let _ = bot_data.bot.add_reaction(
//                 message.channel_id,
//                 message.id,
//                 discord::model::ReactionEmoji::Unicode(String::from("✅"))
//             );

//             let _ = bot_data.bot.add_reaction(
//                 message.channel_id,
//                 message.id,
//                 discord::model::ReactionEmoji::Unicode(String::from("❎"))
//             );
    
//             bot_data.wait_for_reaction.push(general_behaviour::WaitingForReaction::AddMovieToWatched(message.id, movie.clone()));
//         }
//     } else {
//         self::message_failed_to_send_error(bot_data);
//     }
// }

/**
 * Sends an information message, that the movie was not added to the watched status
 */
pub fn movie_not_added_to_watched_information(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::movie_not_added_to_watched_information failed.").channel_id, 
        "",
        |embed| embed
        .title("Status nicht geändert")
        .description("Der Film wurde nicht zum Status 'Watched' hinzugefügt. Bitte denke daran, den Film später manuell hinzuzufügen, falls er geschaut wurde.")
        .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Sends the info message
 */
pub fn info(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::info failed.").channel_id, 
        "",
        |embed| embed
        .fields(|builder| builder
            .field("Author", "Jan Bechtold", true)
            .field("Aktuelle Version", crate::VERSION, false)
        )
        .color(crate::COLOR_BOT)
    );
}

/**
 * Sends an information message, that another user is currently adding a movie and the user
 * should wait for the other user to finish, and try it later.
 */
pub fn another_user_is_adding_a_movie_information(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::another_user_is_adding_a_movie_information failed.").channel_id, 
        "",
        |embed| embed
        .title("Anderer Nutzer fügt gerade einen Film hinzu")
        .description("Ein anderer Nutzer ist gerade dabei einen Film hinzuzufügen. Bitte warte mit deiner Anfrage bis der Nutzer den Vorgang abgeschlossen hat und versuche es dann erneut.")
        .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Sends an error message explaining the user why the data could not be stored
 */
pub fn read_store_data_error(bot_data: &crate::BotData, error: serde_json::Error) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::read_store_data_error failed.").channel_id,
        "",
        |embed| embed
            .title("Fehler beim Speichern")
            .description(
                format!(
                    "Beim Speichern oder Laden der Daten ist ein Fehler aufgetreten. Folgende Fehlermeldung kann ich dir geben:
                    `{} in line {}, column {}`
                    
                    Bitte verständige den Admin. Sollte der Fehler auch bei einem weiteren Versuch bestehen, können die Daten nicht abgespeichert oder geladen werden.",
                    match error.classify() {
                        serde_json::error::Category::Io => "IO Error: Failed to read or write bytes on an IO stream",
                        serde_json::error::Category::Syntax => "Syntax Error: Input is not syntactically correct JSON",
                        serde_json::error::Category::Data => "Data Error: Input data is semantically incorrect",
                        serde_json::error::Category::Eof => "End of file Error: File end came unexpected",
                    },
                    error.line(),
                    error.column()
                )
                .as_str()
            )
            .color(crate::COLOR_ERROR)
    );
}

/**
 * Sends an error message explaining the user, that the file could not be opened
 */
pub fn open_file_error(bot_data: &crate::BotData, error: std::io::Error) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::open_file_error failed.").channel_id,
        "",
        |embed| embed
            .title("Fehler beim Öffnen der Speicherdatei")
            .description(
                format!(
                    "Beim Öffnen der Datei zum Speichern/Lesen der Daten ist ein Fehler aufgetreten. Folgende Fehlermeldung kann ich dir geben:
                    `{:#?}`
                
                    Bitte verständige den Admin. Sollte der Fehler auch bei einem weiteren Versuch bestehen, kann es sein, dass die Daten nicht eingelesen oder gespeichert werden können.",
                    error
                )
                .as_str()
            )
            .color(crate::COLOR_ERROR)
        );
}

/**
 * Sends an error message explaining the user, that during the writing of the file an error occured.
 */
pub fn write_error(bot_data: &crate::BotData, error: std::io::Error) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::write_error failed.").channel_id,
        "",
        |embed| embed
            .title("Fehler beim Öffnen der Speicherdatei")
            .description(
                format!(
                    "Beim Schreiben der Datei zum Speichern der Daten ist ein Fehler aufgetreten. Folgende Fehlermeldung kann ich dir geben:
                    `{:#?}`
                
                    Bitte verständige den Admin. Sollte der Fehler auch bei einem weiteren Versuch bestehen, kann es sein, dass die Daten nicht gespeichert werden können.",
                    error
                )
                .as_str()
            )
            .color(crate::COLOR_ERROR)
        );
}

/**
 * Shows an information message to the user stating, that the data has been saved successfully
 */
pub fn data_saved_successfully(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::data_saved_successfully failed.").channel_id,
        "",
        |embed| embed
            .title("Daten erfolgreich gespeichert")
            .description("Meine Daten wurden erfolgreich in die Speicherdatei geschrieben.")
            .color(crate::COLOR_SUCCESS)
        );
}

/**
 * Tells the user, that the adding of the movie took too long, which is why it timed out
 */
pub fn adding_movie_timed_out_information(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::adding_movie_timed_out_information failed.").channel_id,
        "",
        |embed| embed
            .title("Zeitüberschreitung beim Hinzufügen")
            .description("Das Hinzufügen hat leider zu lange gedauert. Um andere Nutzer nicht beim Hinzufügen zu blockieren, wird das Hinzufügen nach 30 Sekunden automatisch beendet.")
            .color(crate::COLOR_INFORMATION)
        );
}

/**
 * Sends an information message about how many movies with watch list status the user has added 
 */
pub fn current_user_movie_count(bot_data: &crate::BotData, current_movie_count: usize) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_message::current_user_movie_count failed.").channel_id,
        "",
        |embed| embed
            .title("Aktuelle Filmanzahl")
            .description(
                format!("Du hast aktuell `{}` {} hinzugefügt. Du darfst maximal `{}` {} hinzufügen.",
                    current_movie_count,
                    if current_movie_count == 1 { "Film" } else { "Filme" },
                    bot_data.movie_limit_per_user,
                    if bot_data.movie_limit_per_user == 1 { "Film" } else { "Filme" }
                )
                .as_str()
            )
            .color(crate::COLOR_INFORMATION)
        );
}