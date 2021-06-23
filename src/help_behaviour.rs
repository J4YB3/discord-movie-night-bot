use crate::COLOR_INFORMATION;

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
    `info`
    `prefix`
    `quit`
    `save`
    
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
    `close_movie_vote`
    `close_vote`
    `create_vote`
    `movie_vote_limit`
    `random_movie_vote`
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
    Mit dem Sortierparameter `id` wird die Liste nach ID sortiert.
    Mit dem Sortierparameter `user` wird die Liste nach Nutzer sortiert, anschließend nach ID.
    Wird der Parameter weggelassen wird die Liste in beliebiger Reihenfolge angezeigt.
    
    **Nutzung**
    !watch_list <Optional: Sortierung>
    
    **Beispiel**
    !watch_list
    !watch_list user
    !watch_list id
    
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
    "Sendet deine bestehende Abstimmung erneut. Wenn der Parameter leer gelassen wird, wird deine eigene
    Abstimmung erneut gesendet, sofern du eine besitzt. Wenn du einen Nutzer mit dem @-Zeichen angibst, wird
    die Abstimmung des angegebenen Users erneut gesendet, sofern dieser eine besitzt.
    
    **Nutzung**
    !send_vote <Optional: @AndererUser>
    
    **Beispiel**
    !send_vote
    !send_vote @J4YB3
    
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
 * Shows help on the movie_vote_limit command
 */
pub fn show_help_movie_vote_limit(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_movie_vote_limit function failed.");

    let help_str =
    "Zeigt oder setzt die maximale Anzahl der Filme, die zufällig für eine neue Filmabstimmung ausgesucht werden. 
    Dieses Kommando kann nur von Administratoren genutzt werden.
    
    **Nutzung**
    !movie_vote_limit <Optional: positive ganze Zahl>
    
    **Beispiel**
    !movie_vote_limit
    !movie_vote_limit 5
    
    **Aliase**
    `movie_vote_limit`, `mvl`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Movie vote limit - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the random_movie_vote command
 */
pub fn show_help_random_movie_vote(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_random_movie_vote function failed.");

    let help_str =
    "Erstellt eine neue Filmabstimmung mit zufälligen Filmen aus der Filmliste. Wenn bereits eine Filmabstimmung existiert,
    wird diese erneut in den Kanal gesendet.
    
    Wenn eine positive Zahl als Parameter 
    angegeben wird, werden so viele Filme wie angegeben zur Abstimmung ausgewählt. Ansonsten wird das gesetzte Limit
    benutzt.
    
    **Nutzung**
    !random_movie_vote <Optional: positive ganze Zahl>
    
    **Beispiel**
    !random_movie_vote
    !random_movie_vote 5
    
    **Aliase**
    `random_movie_vote`, `rmv`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Random movie vote - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the close_movie_vote command
 */
pub fn show_help_close_movie_vote(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_close_movie_vote function failed.");

    let help_str =
    "Schließt eine bestehende Filmabstimmung, zeigt den Gewinner an, schickt den Watch-Link in den Chat und fragt,
    ob der gewählte Film direkt als Status 'watched' markiert werden soll.
    
    **Nutzung**
    !close_movie_vote
    
    **Beispiel**
    !close_movie_vote
    
    **Aliase**
    `close_movie_vote`, `cmv`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Close movie vote - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

/**
 * Shows help on the info command
 */
pub fn show_help_info(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_info function failed.");

    let help_str =
    "Zeigt Informationen über den Bot an.
    
    **Nutzung**
    !info
    
    **Beispiel**
    !info
    
    **Aliase**
    `info`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Info - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}

pub fn show_help_save(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_info function failed.");

    let help_str =
    "Speichert alle Daten des Bots in die Speicherdatei.
    
    **Nutzung**
    !save
    
    **Beispiel**
    !save
    
    **Aliase**
    `save`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Save - Hilfe").description(help_str).color(COLOR_INFORMATION)
    );
}