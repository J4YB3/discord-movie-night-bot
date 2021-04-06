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