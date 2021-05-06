use std::{fmt, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    AddMovie(String),
    RemoveMovieByTitle(String),
    RemoveMovieById(u32),
    EditMovie(u32, String),
    ShowWatchlist,
    Help(SimpleCommand),
    Prefix(char),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseCommandError {
    NoCommand,
    UnknownCommand,
    NoArgumentsForAdd,
    NoArgumentsForRemove,
    NotEnoughArgumentsForEdit,
    WrongArgumentsForEdit,
    NoArgumentsForPrefix,
    PrefixIsNotAChar,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimpleCommand {
    General,
    Quit,
    Add,
    Remove,
    Edit,
    Show,
    Help,
    Prefix,
    Unknown(String),
}

impl From<&str> for SimpleCommand {
    fn from(s: &str) -> Self {
        match s {
            "" => Self::General,
            QUIT => Self::Quit,
            ADD_MOVIE | ADD_MOVIE_SHORT => Self::Add,
            REMOVE_MOVIE | REMOVE_MOVIE_SHORT => Self::Remove,
            EDIT_MOVIE | EDIT_MOVIE_SHORT => Self::Edit,
            SHOW_WATCH_LIST | SHOW_WATCH_LIST_SHORT => Self::Show,
            HELP | HELP_SHORT => Self::Help,
            PREFIX => Self::Prefix,
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
            Self::Edit => write!(f, "{}", EDIT_MOVIE),
            Self::Show => write!(f, "{}", SHOW_WATCH_LIST),
            Self::Help => write!(f, "{}", HELP),
            Self::Prefix => write!(f, "{}", PREFIX),
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
            SHOW_WATCH_LIST | SHOW_WATCH_LIST_SHORT => Self::ShowWatchlist,
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
            EDIT_MOVIE | EDIT_MOVIE_SHORT => {
                // first argument should be u32, second should be the new title
                if arguments.len() <= 2 {
                    return Err(ParseCommandError::NotEnoughArgumentsForEdit);
                }

                let (id, title) = arguments.split_at(1);
                // We know arguments has at least 2 elements and we split the first off into id.
                // Thus, id has exactly one element at [0].
                let id = id[0];
                // And title has at least one element.
                let title = title.join(" ");

                if let Ok(n) = id.parse::<u32>() {
                    Self::EditMovie(n, title)
                } else {
                    return Err(ParseCommandError::WrongArgumentsForEdit);
                }
            },
            PREFIX => {
                // there should be only one argument, all others will be ignored
                // 
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
            Self::EditMovie(id, title) => format!("{}{} {} {}", bot_data.custom_prefix, EDIT_MOVIE, id, title),
            Self::ShowWatchlist => format!("{}{}", bot_data.custom_prefix, SHOW_WATCH_LIST),
            Self::Help(sc) => format!("{}{} {}", bot_data.custom_prefix, HELP, sc),
            Self::Prefix(new_prefix) => format!("{}{} {}", bot_data.custom_prefix, PREFIX, new_prefix),
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
pub const SHOW_WATCH_LIST: &str = "watch_list"; // !watch_list | Shows the full watch list
pub const SHOW_WATCH_LIST_SHORT: &str = "wl"; // !wl | Short form for watch_list
pub const HELP: &str = "help"; // !help | Shows a list of available commandsa
pub const HELP_SHORT: &str = "h";
pub const PREFIX: &str = "prefix"; // !prefix <char> | Sets a custom prefix. Must be a single character