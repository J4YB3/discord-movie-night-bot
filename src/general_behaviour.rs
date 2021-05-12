use crate::COLOR_INFORMATION;

/**
 * Takes a timestamp from the chrono package and converts it to german date format,
 * translating the english weekday to german in the process.
 */
pub fn timestamp_to_string(timestamp: &chrono::DateTime<chrono::FixedOffset>) -> String {
    let date_format = timestamp.format("%d.%m.%Y");
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
    `quit`
    
    **Filme**
    `add_movie`
    `edit_movie`
    `remove_movie`
    `watch_list`";

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
    "Fügt einen Film zur Filmliste hinzu.
    
    **Nutzung**
    !add_movie <Filmtitel>
    
    **Beispiel**
    !add_movie Forrest Gump
    
    **Aliase**
    `add_movie`, `am`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Add movie - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the edit_movie (em) command
 */
pub fn show_help_edit_movie(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_help function failed.");

    let help_str =
    "Lässt dich den Titel eines zuvor hinzugefügten Films ändern. Um die ID eines Films herauszufinden, benutze das Kommando !watch_list.
    
    **Nutzung**
    !edit_movie <ID> <Neuer Titel>
    
    **Beispiel**
    !edit_movie 3 Star Wars: Episode IV - A New Hope
    !edit_movie 0003 Interstellar
    
    **Aliase**
    `edit_movie`, `em`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Edit movie - Hilfe").description(help_str).color(COLOR_INFORMATION)
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
 * Sets a new custom prefix for all commands
 */
pub fn set_new_prefix(bot_data: &mut crate::BotData, new_prefix: char) {
    let message = bot_data.message.as_ref().expect("Passing message to set_new_prefix function failed.");

    bot_data.custom_prefix = new_prefix;

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Neuer Präfix").description(
            format!(
                "Der Präfix für alle Kommandos wurde zu ``{}`` geändert.
                Bitte benutze nur noch diesen Präfix um auf den Bot zuzugreifen. Der zuvor genutzte Präfix ist nun nicht mehr verfügbar.", 
                new_prefix
            ).as_str()
        ).color(COLOR_INFORMATION)
    );
}