pub static DISCORD_TOKEN: &'static str = match option_env!("DISCORD_TOKEN") {
    Some(value) => value,
    None => "",
};
pub static TMDB_API_KEY: &'static str = match option_env!("TMDB_API_KEY") {
    Some(value) => value,
    None => "",
};
