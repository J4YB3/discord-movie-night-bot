use crate::COLOR_INFORMATION;
use crate::movie_behaviour::is_user_administrator;
use regex::Regex;

#[derive(Clone, Debug)]
pub enum WaitingForReaction {
    AddMovie(discord::model::MessageId, crate::movie_behaviour::WatchListEntry),
    Vote(discord::model::MessageId),
}

/**
 * Takes a timestamp from the chrono package and converts it to german date format,
 * translating the english weekday to german in the process.
 */
pub fn timestamp_to_string(timestamp: &chrono::DateTime<chrono::FixedOffset>, include_weekday: bool) -> String {
    let date_format = timestamp.format("%d.%m.%Y");
    
    if include_weekday {
        let day = match format!("{}", timestamp.format("%A")).as_str() {
            "Monday" => "Montag",
            "Tuesday" => "Dienstag",
            "Wednesday" => "Mittwoch",
            "Thursday" => "Donnerstag",
            "Friday" => "Freitag",
            "Saturday" => "Samstag",
            "Sunday" => "Sonntag",
            _ => "",
        };

        format!("{}, {}", day, date_format).to_string()
    } else {
        format!("{}", date_format).to_string()
    }
}

/**
 * Takes a movie release date in the format yyyy-mm-dd and parses it to the chrono datetime format
 */
pub fn parse_tmdb_release_date(tmdb_date: String) -> Result<chrono::DateTime<chrono::FixedOffset>, String> {
    let date_with_utc = tmdb_date.clone() + " 12:00:00.000 +0000";
    if let Ok(datetime) = chrono::DateTime::parse_from_str(date_with_utc.as_str(), "%Y-%m-%d %H:%M:%S%.3f %z") {
        Ok(datetime)
    } else {
        Err("Parsing of movie release date failed".to_string())
    }
}

/**
 * Takes an IMDb link and extracts the IMDb ID
 */
pub fn parse_imdb_link_id(hyperlink: String) -> Option<String> {
    // Example link: https://www.imdb.com/title/tt0816692/?ref_=fn_al_tt_2
    if let Some(regex_match) = Regex::new(r"[a-z][a-z][0-9]+").unwrap().find(hyperlink.as_str()) {
        Some(regex_match.as_str().to_string())
    } else {
        None
    }
}

/**
 * Takes the budget of a movie and formats it to easy read format (169 mio.)
 */
pub fn format_budget(budget: u64) -> String {
    let budget_string = format!("{}", budget).to_string();
    // 169000000 = 169 mio = 169.000.000

    if budget == 0 {
        return String::from("Unbekannt");
    } else if budget_string.len() <= 3 {
        return budget_string.clone();
    } else {
        let (first, _) = budget_string.split_at(3);
        return match budget_string.len() {
            4..=6 => String::from(format!("{} k", first).as_str()),
            7..=9 => String::from(format!("{} mio.", first).as_str()),
            10..=12 => String::from(format!("{} mrd.", first).as_str()),
            _ => budget_string.clone()
        };
    }
}

/**
 * Returns true if the given ReactionEmoji enum equals the unicode emoji
 */
pub fn reaction_emoji_equals(reaction_emoji: &discord::model::ReactionEmoji, unicode: String) -> bool {
    reaction_emojis_equal(reaction_emoji, &discord::model::ReactionEmoji::Unicode(unicode))
}

/**
 * Returns true if two ReactionEmoji enum values are equal
 */
pub fn reaction_emojis_equal(first: &discord::model::ReactionEmoji, second: &discord::model::ReactionEmoji) -> bool {
    if let discord::model::ReactionEmoji::Unicode(first_string) = first {
        if let discord::model::ReactionEmoji::Unicode(second_string) = second {
            first_string == second_string
        } else {
            false
        }
    } else {
        if let discord::model::ReactionEmoji::Unicode(_) = second {
            false
        } else {
            if let discord::model::ReactionEmoji::Custom{name: _, id: first_id} = first {
                if let discord::model::ReactionEmoji::Custom{name: _, id: second_id} = second {
                    first_id == second_id
                } 
                // If the second emoji is none of its two enum options return false
                else {
                    false
                }
            } 
            // If the first emoji is none of its two enum options return false
            else {
                false
            }
        }
    }
}

/**
 * Returns the link for the no-image-available image
 */
pub fn get_no_image_available_url() -> &'static str {
    "https://upload.wikimedia.org/wikipedia/commons/thumb/6/65/No-Image-Placeholder.svg/330px-No-Image-Placeholder.svg.png"
}

/**
 * Returns the link to the TMDb logo for attribution
 */
pub fn get_tmdb_attribution_icon_url() -> &'static str {
    "https://www.themoviedb.org/assets/2/v4/logos/312x276-primary-green-74212f6247252a023be0f02a5a45794925c3689117da9d20ffe47742a665c518.png"
}

/**
 * Show a basic embedded message containing all available commands grouped by functionality
 */
pub fn show_help(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help function failed.");

    let help_str =
    "Einige Kommandos besitzen Aliase, die kürzer als das normale Kommando sind.
    Für mehr Informationen zu jedem Kommando, benutze bitte !help <Kommando>
    **Beispiel**: !help watch_list

    **Allgemein**
    `help`
    `prefix`
    `quit`
    
    **Filme**
    `add_movie`
    `history`
    `movie_limit`
    `remove_movie`
    `search_movie`
    `set_status`
    `show_movie`
    `unavailable`
    `watched`
    `watch_list`
    
    **Abstimmungen**
    `close_vote`
    `create_vote`
    `send_vote`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Verfügbare Kommandos").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the help command
 */
pub fn show_help_help(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_help function failed.");

    let help_str =
    "Zeigt eine allgemeine Hilfe, sowie eine Liste aller Kommandos an.
    
    **Nutzung**
    !help
    !help <Kommando>
    
    **Beispiel**
    !help
    !help add_movie
    
    **Aliase**
    `help`, `h`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Help - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the quit command
 */
pub fn show_help_quit(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_quit function failed.");

    let help_str =
    "Beendet den Bot und speichert alle relevanten Daten in Dateien auf dem Host-Rechner.
    
    **Nutzung**
    !quit
    
    **Beispiel**
    !quit
    
    **Aliase**
    `quit`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Quit - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Show an embedded message containing information for the add_movie (am) command
 */
pub fn show_help_add_movie(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_add_movie function failed.");

    let help_str = 
    "Sucht nach einem Film auf TMDb und fügt ihn zur Liste hinzu, wenn dieser vom Nutzer bestätigt wird.
    
    **Nutzung**
    !add_movie <Filmtitel | IMDb Link>
    
    **Beispiel**
    !add_movie Forrest Gump
    !add_movie https://www.imdb.com/title/tt9760504/?ref_=fn_al_tt_1
    
    **Aliase**
    `add_movie`, `am`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Add movie - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the remove_movie (rm) command
 */
pub fn show_help_remove_movie(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_remove_movie function failed.");

    let help_str =
    "Ermöglicht es dir einen Film von der Filmliste zu entfernen.
    
    **Nutzung**
    !remove_movie <ID>
    !remove_movie <Filmtitel>
    
    **Beispiel**
    !remove_movie 3
    !remove_movie Interstellar
    
    **Aliase**
    `remove_movie`, `rm`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Remove movie - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the watchlist (wl) command
 */
pub fn show_help_watchlist(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_watchlist function failed.");

    let help_str =
    "Zeigt die Filmliste an.
    Mit dem Sortierparameter `random` wird die Liste in beliebiger Reihenfolge angezeigt.
    Mit dem Sortierparameter `user` wird die Liste nach Nutzer sortiert, anschließend nach ID.
    Wird der Parameter weggelassen wird die Liste in beliebiger Reihenfolge angezeigt.
    
    **Nutzung**
    !watch_list <Optional: Sortierung>
    
    **Beispiel**
    !watch_list
    !watch_list user
    !watch_list random
    
    **Aliase**
    `watch_list`, `wl`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Watch list - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the prefix command
 */
pub fn show_help_prefix(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_prefix function failed.");

    let help_str =
    "Setzt einen neuen benutzerdefinierten Präfix für alle Kommandos. Es sind nur einzelne Zeichen als Präfix erlaubt.
    
    **Nutzung**
    !prefix <Neuer Präfix>
    
    **Beispiel**
    !prefix _
    
    **Aliase**
    `prefix`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Watch list - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the history command
 */
pub fn show_help_history(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_history function failed.");

    let help_str =
    "Zeigt einen Verlauf aller bereits geschauten Filme an, sowie Filme die den Status *gelöscht* haben.
    Mit dem Sortierparameter `date` wird die Liste nach Datum sortiert angezeigt.
    Mit dem Sortierparameter `user` wird die Liste nach Nutzer sortiert, anschließend nach Datum.
    Wird der Parameter weggelassen wird die Liste nach Datum sortiert angezeigt.
    
    **Nutzung**
    !history <Optional: Sortierung>
    
    **Beispiel**
    !history
    !history date
    !history user
    
    **Aliase**
    `history`, `hs`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: History - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the set_status command
 */
pub fn show_help_set_status(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_set_status function failed.");

    let help_str =
    "Setzt den Status eines Films. Der erste Wert gibt die ID des Films an. Der zweite den neuen Status.
    Folgende Status sind verfügbar: `NotWatched`, `Watched`, `Unavailable`, `Rewatch`, `Removed`
    Groß- und Kleinschreibung wird bei den Status ignoriert.
    Die Status `Watched` und `Removed` führen dazu, dass ein Film von der Watch list entfernt wird.
    Der Status `Unavailable` wird bei der Zählung der Filme pro Nutzer ignoriert.
    
    **Nutzung**
    !set_status <ID> <Status>
    
    **Beispiel**
    !set_status 3 Watched
    !set_status 0010 unavailable
    !set_status 20 REMOVED
    
    **Aliase**
    `set_status`, `st`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Set Status - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the unavailable command
 */
pub fn show_help_set_status_unavailable(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_set_status_unavailable function failed.");

    let help_str =
    "Setzt den Status eines Films mit der ID direkt zu `Unavailable`
    Der Status `Unavailable` wird bei der Zählung der Filme pro Nutzer ignoriert.
    
    **Nutzung**
    !unavailable <ID>
    
    **Beispiel**
    !unavailable 3
    !unavailable 0010
    
    **Aliase**
    `unavailable`, `un`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Unavailable - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the watched command
 */
pub fn show_help_set_status_watched(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_set_status_watched function failed.");

    let help_str =
    "Setzt den Status eines Films mit der ID direkt zu `Watched`
    Der Status `Watched` führt dazu, dass der Film im Verlauf angezeigt wird und von der Watch Liste verschwindet.
    Falls der Film an einem anderen Datum als dem aktuellen geschaut wurde, kann durch den zweiten Parameter ein Datum
    im Format TT.MM.JJJJ angegeben werden.
    
    **Nutzung**
    !watched <ID> <Optional: Datum (TT.MM.JJJJ)>
    
    **Beispiel**
    !watched 10
    !watched 0002 15.05.2021
    
    **Aliase**
    `watched`, `wa`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Watched - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the show_movie command
 */
pub fn show_help_show_movie(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_show_movie function failed.");

    let help_str =
    "Zeigt Informationen zu einem Film von der Watch List oder der History an.
    
    **Nutzung**
    !show_movie <ID>
    !show_movie <Titel>
    
    **Beispiel**
    !show_movie 10
    !show_movie Ni no Kuni
    
    **Aliase**
    `show_movie`, `sm`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Show Movie - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Show an embedded message containing information for the search_movie (search) command
 */
pub fn show_help_search_movie(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_search_movie function failed.");

    let help_str = 
    "Sucht nach einem Film auf TMDb und zeigt seine Informationen an, ohne den Film zur Liste hinzuzufügen.
    
    **Nutzung**
    !search_movie <Filmtitel | IMDb Link>
    
    **Beispiel**
    !search_movie Forrest Gump
    !search_movie https://www.imdb.com/title/tt9760504/
    
    **Aliase**
    `search_movie`, `search`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Search movie - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the help command
 */
pub fn show_help_create_vote(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_create_vote function failed.");

    let help_str =
    "Erstellt eine neue Abstimmung, die sowohl Filme, als auch generelle Optionen enthalten kann. 
    Leerzeichen am Anfang und Ende der einzelnen Optionen werden ignoriert.
    
    **Nutzung**
    !create_vote <Titel>|<Liste von Optionen getrennt durch '|'>
    
    **Beispiel**
    !create_vote Das hier ist eine Abstimmung|Option 1|Option 2
    !create_vote Test|Option 1 |  Option 2   | Option 3
    
    **Aliase**
    `create_vote`, `cv`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Create vote - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the send_vote command
 */
pub fn show_help_send_vote(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_send_vote function failed.");

    let help_str =
    "Sendet deine bestehende Abstimmung erneut, sofern du eine hast.
    
    **Nutzung**
    !send_vote
    
    **Beispiel**
    !send_vote
    
    **Aliase**
    `send_vote`, `sv`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Send vote - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the close_vote command
 */
pub fn show_help_close_vote(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_close_vote function failed.");

    let help_str =
    "Beendet deine eigene Abstimmung, sofern du eine hast.
    
    **Nutzung**
    !close_vote
    
    **Beispiel**
    !close_vote
    
    **Aliase**
    `close_vote`, `xv`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Close vote - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the movie_limit command
 */
pub fn show_help_movie_limit(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_movie_limit function failed.");

    let help_str =
    "Zeigt oder setzt die maximale Anzahl der Filme, die jeder Nutzer hinzufügen darf. 
    Dieses Kommando kann nur von Administratoren genutzt werden.
    
    **Nutzung**
    !movie_limit <Optional: positive ganze Zahl>
    
    **Beispiel**
    !movie_limit
    !movie_limit 5
    
    **Aliase**
    `movie_limit`, `ml`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Movie limit - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Sets a new custom prefix for all commands
 */
pub fn set_new_prefix(bot_data: &mut crate::BotData, new_prefix: char) {
    let message = bot_data.message.as_ref().expect("Passing message to set_new_prefix function failed.");

    if is_user_administrator(bot_data, message.author.id) {
        bot_data.custom_prefix = new_prefix;

        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed.title(":information_source: Neuer Präfix").description(
                format!(
                    "Der Präfix für alle Kommandos wurde zu `{}` geändert.
                    Bitte benutze nur noch diesen Präfix um auf den Bot zuzugreifen. Der zuvor genutzte Präfix ist nun nicht mehr verfügbar.", 
                    new_prefix
                ).as_str()
            ).color(COLOR_INFORMATION)
        );
    } else {
        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed.title(":information_source: Rechte nicht ausreichend").description(
                format!(
                    "Du benötigst Administrator-Rechte um den Präfix für diesen Bot zu ändern."
                ).as_str()
            ).color(COLOR_INFORMATION)
        );
    }
}