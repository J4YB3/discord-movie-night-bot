/**
 * Constructs the command with the default prefix '!'
 */
pub fn construct(command: &str) -> String {
    String::from("!") + command
}

/**
 * Constructs the command with a custom prefix
 */
pub fn construct_custom(command: &str, custom_prefix: String) -> String {
    custom_prefix + command
}

// Command, Usage | Description
pub const QUIT: &str = "quit"; // !quit | Quits the bot and saves all changes
pub const ADD_MOVIE: &str = "add_movie"; // !add_movie <title> | Adds a movie to the watch list
pub const ADD_MOVIE_SHORT: &str = "am"; // !am <title> | Short form for add_movie
pub const REMOVE_MOVIE: &str = "remove_movie"; // !remove_movie <title|id> | Removes a movie by id or by title from the watch list
pub const REMOVE_MOVIE_SHORT: &str = "rm"; // !rm <title|id> | Short form for remove_movie
pub const EDIT_MOVIE: &str = "edit_movie"; // !edit_movie <id> <new_title> | Changes the movie specified by its id to a new title
pub const EDIT_MOVIE_SHORT: &str = "em"; // !em <id> <new_title> | Short form for edit_movie
pub const SHOW_WATCH_LIST: &str = "watch_list"; // !watch_list | Shows the full watch list
pub const SHOW_WATCH_LIST_SHORT: &str = "wl"; // !wl | Short form for watch_list

pub const COLOR_ERROR: u64 = 0xff0000; // red
pub const COLOR_SUCCESS: u64 = 0x7ef542; // green
pub const COLOR_WARNING: u64 = 0xf5d442; // yellow