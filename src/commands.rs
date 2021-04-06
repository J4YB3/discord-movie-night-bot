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

pub const QUIT: &str = "quit";
pub const ADD_MOVIE: &str = "add_movie";
pub const ADD_MOVIE_SHORT: &str = "am";
pub const REMOVE_MOVIE: &str = "remove_movie";
pub const REMOVE_MOVIE_SHORT: &str = "rm";
pub const EDIT_MOVIE: &str = "edit_movie";
pub const EDIT_MOVIE_SHORT: &str = "em";