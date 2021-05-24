use std::{fmt, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    AddMovie(String),
    RemoveMovieByTitle(String),
    RemoveMovieById(u32),
    ShowWatchlist(String),
    Help(SimpleCommand),
    Prefix(char),
    History(String),
    SetStatus(u32, String),
    Unavailable(u32),
    Watched(u32, String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseCommandError {
    NoCommand,
    UnknownCommand,
    NoArgumentsForAdd,
    NoArgumentsForRemove,
    NoArgumentsForPrefix,
    PrefixIsNotAChar,
    WrongArgumentForWatchList,
    WrongArgumentForHistory,
    NotEnoughArgumentsForStatus,
    WrongArgumentsForStatus,
    WrongArgumentForUnavailable,
    NoArgumentForUnavailable,
    NotEnoughArgumentsForWatched,
    WrongArgumentsForWatched,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimpleCommand {
    General,
    Quit,
    Add,
    Remove,
    Show,
    Help,
    Prefix,
    History,
    Status,
    Unavailable,
    Watched,
    Unknown(String),
}

impl From<&str> for SimpleCommand {
    fn from(s: &str) -> Self {
        match s {
            "" => Self::General,
            QUIT => Self::Quit,
            ADD_MOVIE | ADD_MOVIE_SHORT => Self::Add,
            REMOVE_MOVIE | REMOVE_MOVIE_SHORT => Self::Remove,
            SHOW_WATCH_LIST | SHOW_WATCH_LIST_SHORT => Self::Show,
            HELP | HELP_SHORT => Self::Help,
            PREFIX => Self::Prefix,
            SHOW_HISTORY | SHOW_HISTORY_SHORT => Self::History,
            SET_STATUS | SET_STATUS_SHORT => Self::Status,
            SET_STATUS_UNAVAILABLE | SET_STATUS_UNAVAILABLE_SHORT => Self::Unavailable,
            SET_STATUS_WATCHED | SET_STATUS_WATCHED_SHORT => Self::Watched,
            st => Self::Unknown(String::from(st)),
        }
    }
}

impl fmt::Display for SimpleCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::General => write!(f, ""),
            Self::Quit => write!(f, "{}", QUIT),
            Self::Add => write!(f, "{}", ADD_MOVIE),
            Self::Remove => write!(f, "{}", REMOVE_MOVIE),
            Self::Show => write!(f, "{}", SHOW_WATCH_LIST),
            Self::Help => write!(f, "{}", HELP),
            Self::Prefix => write!(f, "{}", PREFIX),
            Self::History => write!(f, "{}", SHOW_HISTORY),
            Self::Status => write!(f, "{}", SET_STATUS),
            Self::Unavailable => write!(f, "{}", SET_STATUS_UNAVAILABLE),
            Self::Watched => write!(f, "{}", SET_STATUS_WATCHED),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // separate command from its arguments
        let input = s.split_whitespace().collect::<Vec<&str>>();
        if input.len() <= 0 {
            return Err(ParseCommandError::NoCommand);
        }

        // see https://doc.rust-lang.org/std/primitive.slice.html#method.split_at
        let (command, arguments) = input.split_at(1);

        // We know the input contains at least one element and we split off the first element into command.
        // Thus, command has exactly one element at [0]. For comparison the prefix needs to be removed,
        // therefore we use the slice from index 1 to the end
        Ok(match &command[0][1..] {
            QUIT => Self::Quit,
            HELP | HELP_SHORT => Self::Help(SimpleCommand::from(arguments.join(" ").as_str())),
            SHOW_WATCH_LIST | SHOW_WATCH_LIST_SHORT => {
                // Only one argument is expected. All others will be ignored
                let mut argument = "".to_string();
                if arguments.len() > 0 {
                    argument = arguments[0].to_lowercase();
                }

                if argument != "random" && argument != "user" && argument != "" {
                    return Err(ParseCommandError::WrongArgumentForWatchList);
                }

                Self::ShowWatchlist(argument)
            },
            ADD_MOVIE | ADD_MOVIE_SHORT => {
                let title = arguments.join(" ");
                if title.is_empty() {
                    return Err(ParseCommandError::NoArgumentsForAdd); // Returning bypasses Ok-wrapping.
                }

                Self::AddMovie(title)
            },
            REMOVE_MOVIE | REMOVE_MOVIE_SHORT => {
                let argument = arguments.join(" ");
                if argument.is_empty() {
                    return Err(ParseCommandError::NoArgumentsForRemove);
                }

                // Try to parse the first argument to u32. If that is not possible assume it's a title instead of an id.
                if let Ok(n) = argument.parse::<u32>() {
                    Self::RemoveMovieById(n)
                } else {
                    Self::RemoveMovieByTitle(argument)
                }
            },
            PREFIX => {
                // there should be only one argument, all others will be ignored
                if arguments.len() <= 0 {
                    return Err(ParseCommandError::NoArgumentsForPrefix);
                }

                let (new_prefix, _) = arguments.split_at(1);
                let new_prefix = new_prefix[0];

                if let Ok(new_prefix_char) = new_prefix.parse::<char>() {
                    Self::Prefix(new_prefix_char)
                } else {
                    return Err(ParseCommandError::PrefixIsNotAChar);
                }
            },
            SHOW_HISTORY | SHOW_HISTORY_SHORT => {
                // Only one argument expected. Others are ignored
                let mut argument = "".to_string();
                if arguments.len() > 0 {
                    argument = arguments[0].to_lowercase();
                }

                if argument != "date" && argument != "user" && argument != "" {
                    return Err(ParseCommandError::WrongArgumentForHistory);
                }

                Self::History(argument)
            },
            SET_STATUS | SET_STATUS_SHORT => {
                // first argument should be u32, second should be the new status
                if arguments.len() < 2 {
                    return Err(ParseCommandError::NotEnoughArgumentsForStatus);
                }

                let (id, status) = arguments.split_at(1);
                // We know arguments has at least 2 elements and we split the first off into id.
                // Thus, id has exactly one element at [0].
                let id = id[0];
                // And status has at least one element.
                let status = status[0];

                if let Ok(n) = id.parse::<u32>() {
                    Self::SetStatus(n, status.to_string())
                } else {
                    return Err(ParseCommandError::WrongArgumentsForStatus);
                }
            },
            SET_STATUS_UNAVAILABLE | SET_STATUS_UNAVAILABLE_SHORT => {
                let argument = arguments.join(" ");
                if argument.is_empty() {
                    return Err(ParseCommandError::NoArgumentForUnavailable);
                }

                // Try to parse the first argument to u32. If that is not possible assume it's a title instead of an id.
                if let Ok(n) = argument.parse::<u32>() {
                    Self::Unavailable(n)
                } else {
                    return Err(ParseCommandError::WrongArgumentForUnavailable);
                }
            },
            SET_STATUS_WATCHED | SET_STATUS_WATCHED_SHORT => {
                // first argument should be u32, second should be the new status
                if arguments.len() < 1 {
                    return Err(ParseCommandError::NotEnoughArgumentsForWatched);
                }

                let (id, arguments) = arguments.split_at(1);
                // We know arguments has at least 1 element and we split the first off into id.
                // Thus, id has exactly one element at [0].
                let id = id[0];
                
                let mut date = "";
                if arguments.len() > 0 {
                    date = arguments[0].trim();
                }

                if let Ok(n) = id.parse::<u32>() {
                    Self::Watched(n, date.to_string())
                } else {
                    return Err(ParseCommandError::WrongArgumentsForWatched);
                }
            }
            _ => return Err(ParseCommandError::UnknownCommand),
        })
    }
}

impl Command {
    pub fn to_string(&self, bot_data: &crate::BotData) -> String {
        match self {
            Self::AddMovie(title) => format!("{}{} {}", bot_data.custom_prefix, ADD_MOVIE, title),
            Self::Quit => format!("{}{}", bot_data.custom_prefix, QUIT),
            Self::RemoveMovieByTitle(title) => format!("{}{} {}", bot_data.custom_prefix, REMOVE_MOVIE, title),
            Self::RemoveMovieById(id) => format!("{}{} {}", bot_data.custom_prefix, REMOVE_MOVIE, id),
            Self::ShowWatchlist(order) => format!("{}{} {}", bot_data.custom_prefix, SHOW_WATCH_LIST, order),
            Self::Help(sc) => format!("{}{} {}", bot_data.custom_prefix, HELP, sc),
            Self::Prefix(new_prefix) => format!("{}{} {}", bot_data.custom_prefix, PREFIX, new_prefix),
            Self::History(order) => format!("{}{} {}", bot_data.custom_prefix, SHOW_HISTORY, order),
            Self::SetStatus(id, status) => format!("{}{} {} {}", bot_data.custom_prefix, SET_STATUS, id, status),
            Self::Unavailable(id) => format!("{}{} {}", bot_data.custom_prefix, SET_STATUS_UNAVAILABLE, id),
            Self::Watched(id, date) => format!("{}{} {} {}", bot_data.custom_prefix, SET_STATUS_WATCHED, id, date),
        }
    }
}

// Command, Usage | Description
pub const QUIT: &str = "quit"; // !quit | Quits the bot and saves all changes
pub const ADD_MOVIE: &str = "add_movie"; // !add_movie <title> | Adds a movie to the watch list
pub const ADD_MOVIE_SHORT: &str = "am"; // !am <title> | Short form for add_movie
pub const REMOVE_MOVIE: &str = "remove_movie"; // !remove_movie <title|id> | Removes a movie by id or by title from the watch list
pub const REMOVE_MOVIE_SHORT: &str = "rm"; // !rm <title|id> | Short form for remove_movie
pub const EDIT_MOVIE: &str = "edit_movie"; // !edit_movie <id> <new_title> | Changes the movie specified by its id to a new title
pub const EDIT_MOVIE_SHORT: &str = "em"; // !em <id> <new_title> | Short form for edit_movie
pub const SHOW_WATCH_LIST: &str = "watch_list"; // !watch_list <optional: order> | Shows the full watch list
pub const SHOW_WATCH_LIST_SHORT: &str = "wl"; // !wl <optional: order> | Short form for watch_list
pub const HELP: &str = "help"; // !help, !help <command> | Shows a list of available commands or help to one specific command
pub const HELP_SHORT: &str = "h"; // !h, !h <command> | Short form for help
pub const PREFIX: &str = "prefix"; // !prefix <char> | Sets a custom prefix. Must be a single character
pub const SHOW_HISTORY: &str = "history"; // !history <optional: order> | Shows a list of all movies that have been watched already or that have the status 'removed'
pub const SHOW_HISTORY_SHORT: &str = "hs"; // !h <optional: order> | Short form for history
pub const SET_STATUS: &str = "set_status"; // !set_status <id> <movie_status> | Sets the status of a movie
pub const SET_STATUS_SHORT: &str = "st"; // !st <id> <movie_status> | Short form for set_status
pub const SET_STATUS_UNAVAILABLE: &str = "unavailable"; // !unavailable <id> | Sets the given movie with id to the unavailable status
pub const SET_STATUS_UNAVAILABLE_SHORT: &str = "un"; // !un <id> | Short form for unavailable
pub const SET_STATUS_WATCHED: &str = "watched"; // !watched <id> <optional: date> | Sets the given movie with id to watched. If date is given the timestamp is set to this date
pub const SET_STATUS_WATCHED_SHORT: &str = "wa"; // !wa <id> <optional: date> | Short form for watched