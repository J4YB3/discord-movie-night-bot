use crate::commands;

/**
 * Show a basic embedded message containing all available commands grouped by functionality
 */
pub fn show_help(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help function failed.");

    let help_str =
    "Some commands have aliases that are shorter than the actual command.
    For more information on each command, use !help <command>
    **Example**: !help watchlist

    **General**
    `help`
    `quit`
    
    **Movies**
    `add_movie`
    `edit_movie`
    `remove_movie`
    `watchlist`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Available commands").description(help_str).color(commands::COLOR_INFORMATION)
    );
}

/**
 * Shows help on the help command
 */
pub fn show_help_help(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_help function failed.");

    let help_str =
    "Shows general help and a list of all commands
    
    **Usage**
    !help
    !help <command>
    
    **Example usage**
    !help
    !help add_movie
    
    **Aliases**
    `help`, `h`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Help - Help").description(help_str).color(commands::COLOR_INFORMATION)
    );
}

/**
 * Shows help on the quit command
 */
pub fn show_help_quit(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_quit function failed.");

    let help_str =
    "Shuts down the bot, saving all data in files.
    
    **Usage**
    !quit
    
    **Example usage**
    !quit
    
    **Aliases**
    `quit`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Quit - Help").description(help_str).color(commands::COLOR_INFORMATION)
    );
}

/**
 * Show an embedded message containing information for the add_movie (am) command
 */
pub fn show_help_add_movie(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_add_movie function failed.");

    let help_str = 
    "Adds a movie to the watch list
    
    **Usage**
    !add_movie <movie_title>
    
    **Example usage**
    !add_movie Forrest Gump
    
    **Aliases**
    `add_movie`, `am`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Add movie - Help").description(help_str).color(commands::COLOR_INFORMATION)
    );
}

/**
 * Shows help on the edit_movie (em) command
 */
pub fn show_help_edit_movie(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_help function failed.");

    let help_str =
    "Lets you edit the title of previously added movies. To get the ID of the movie, use the !watchlist command
    
    **Usage**
    !edit_movie <id> <new_title>
    
    **Example usage**
    !edit_movie 3 Star Wars: Episode IV - A New Hope
    !edit_movie 0003 Interstellar
    
    **Aliases**
    `edit_movie`, `em`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Edit movie - Help").description(help_str).color(commands::COLOR_INFORMATION)
    );
}

/**
 * Shows help on the remove_movie (rm) command
 */
pub fn show_help_remove_movie(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_remove_movie function failed.");

    let help_str =
    "Lets you remove a movie from the watch list.
    
    **Usage**
    !remove_movie <id>
    !remove_movie <movie_title>
    
    **Example usage**
    !remove_movie 3
    !remove_movie Interstellar
    
    **Aliases**
    `remove_movie`, `rm`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Remove movie - Help").description(help_str).color(commands::COLOR_INFORMATION)
    );
}

/**
 * Shows help on the watchlist (wl) command
 */
pub fn show_help_watchlist(bot_data: &crate::BotData) {
    let message = bot_data.message.as_ref().expect("Passing message to show_help_watchlist function failed.");

    let help_str =
    "Shows the watch list sorted by user and then by ID
    
    **Usage**
    !watchlist
    
    **Example usage**
    !watchlist
    
    **Aliases**
    `watchlist`, `wl`";

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed.title(":information_source: Watch list - Help").description(help_str).color(commands::COLOR_INFORMATION)
    );
}