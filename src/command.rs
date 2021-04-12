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
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseCommandError {
    NoCommand,
    UnknownCommand,
    NoArgumentsForAdd,
    NoArgumentsForRemove,
    NotEnoughArgumentsForEdit,
    WrongArgumentForEdit,
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

        let (command, arguments) = input.split_at(1);

        // We know the input contains at least one element and we split off the first element into command.
        // Thus, command has exactly one element at [0].
        Ok(match command[0] {
            QUIT => Self::Quit,
            HELP | HELP_SHORT => Self::Help(SimpleCommand::from(arguments.join(" ").as_str())),
            SHOW_WATCH_LIST | SHOW_WATCH_LIST_SHORT => Self::ShowWatchlist,
            ADD_MOVIE | ADD_MOVIE_SHORT => {
                let title = arguments.join(" ");
                if title.is_empty() {
                    return Err(ParseCommandError::NoArgumentsForAdd); // Returning bypasses Ok-wrapping.
                }

                Self::AddMovie(title)
            }
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
            }
            EDIT_MOVIE | EDIT_MOVIE_SHORT => {
                if arguments.len() <= 2 {
                    return Err(ParseCommandError::NotEnoughArgumentsForEdit);
                }

                let (id, title) = arguments.split_at(1);
                // We know arguments has at least 2 elements and we split the first off into id.
                // Thus, id has exactly one element at [0].
                let id = id[0];
                // And title has at least one element.
                let title = title.join(" ");

                // Id needs to be a u32.
                if let Ok(n) = id.parse::<u32>() {
                    Self::EditMovie(n, title)
                } else {
                    return Err(ParseCommandError::WrongArgumentForEdit);
                }
            }
            _ => return Err(ParseCommandError::UnknownCommand),
        })
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddMovie(title) => write!(f, "!{} {}", ADD_MOVIE, title),
            Self::Quit => write!(f, "!{}", QUIT),
            Self::RemoveMovieByTitle(title) => write!(f, "!{} {}", REMOVE_MOVIE, title),
            Self::RemoveMovieById(id) => write!(f, "!{} {}", REMOVE_MOVIE, id),
            Self::EditMovie(id, title) => write!(f, "!{} {} {}", EDIT_MOVIE, id, title),
            Self::ShowWatchlist => write!(f, "!{}", SHOW_WATCH_LIST),
            Self::Help(sc) => write!(f, "!{} {}", HELP, sc),
        }
    }
}

// Command, Usage | Description
const QUIT: &str = "quit"; // !quit | Quits the bot and saves all changes
const ADD_MOVIE: &str = "add_movie"; // !add_movie <title> | Adds a movie to the watch list
const ADD_MOVIE_SHORT: &str = "am"; // !am <title> | Short form for add_movie
const REMOVE_MOVIE: &str = "remove_movie"; // !remove_movie <title|id> | Removes a movie by id or by title from the watch list
const REMOVE_MOVIE_SHORT: &str = "rm"; // !rm <title|id> | Short form for remove_movie
const EDIT_MOVIE: &str = "edit_movie"; // !edit_movie <id> <new_title> | Changes the movie specified by its id to a new title
const EDIT_MOVIE_SHORT: &str = "em"; // !em <id> <new_title> | Short form for edit_movie
const SHOW_WATCH_LIST: &str = "watch_list"; // !watch_list | Shows the full watch list
const SHOW_WATCH_LIST_SHORT: &str = "wl"; // !wl | Short form for watch_list
const HELP: &str = "help"; // !help | Shows a list of available commandsa
const HELP_SHORT: &str = "h";

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::command::{
        ADD_MOVIE, ADD_MOVIE_SHORT, EDIT_MOVIE, EDIT_MOVIE_SHORT, HELP, HELP_SHORT, QUIT,
        REMOVE_MOVIE, REMOVE_MOVIE_SHORT, SHOW_WATCH_LIST, SHOW_WATCH_LIST_SHORT,
    };

    use super::{Command, ParseCommandError, SimpleCommand};

    const TITLE: &str = "Cars 2";
    const ID: &str = "123";
    const UNKNOWN: &str = "nuke_server";

    #[test]
    fn test_quit() {
        let c = Command::from_str(QUIT);
        assert_eq!(c, Ok(Command::Quit));
    }

    #[test]
    fn test_add_movie_success() {
        let c = Command::from_str(format!("{} {}", ADD_MOVIE, TITLE).as_str()).unwrap();
        assert_eq!(c, Command::AddMovie(String::from(TITLE)));
        let c = Command::from_str(format!("{} {}", ADD_MOVIE_SHORT, TITLE).as_str()).unwrap();
        assert_eq!(c, Command::AddMovie(String::from(TITLE)));
    }

    #[test]
    fn test_remove_movie_by_title() {
        let c = Command::from_str(format!("{} {}", REMOVE_MOVIE, TITLE).as_str()).unwrap();
        assert_eq!(c, Command::RemoveMovieByTitle(String::from(TITLE)));
        let c = Command::from_str(format!("{} {}", REMOVE_MOVIE_SHORT, TITLE).as_str()).unwrap();
        assert_eq!(c, Command::RemoveMovieByTitle(String::from(TITLE)));
    }

    #[test]
    fn test_remove_movie_by_id() {
        let c = Command::from_str(format!("{} {}", REMOVE_MOVIE, ID).as_str()).unwrap();
        assert_eq!(c, Command::RemoveMovieById(ID.parse().unwrap()));
        let c = Command::from_str(format!("{} {}", REMOVE_MOVIE_SHORT, ID).as_str()).unwrap();
        assert_eq!(c, Command::RemoveMovieById(ID.parse().unwrap()));
    }

    #[test]
    fn test_edit_movie() {
        let c = Command::from_str(format!("{} {} {}", EDIT_MOVIE, ID, TITLE).as_str()).unwrap();
        assert_eq!(
            c,
            Command::EditMovie(ID.parse().unwrap(), String::from(TITLE))
        );
        let c =
            Command::from_str(format!("{} {} {}", EDIT_MOVIE_SHORT, ID, TITLE).as_str()).unwrap();
        assert_eq!(
            c,
            Command::EditMovie(ID.parse().unwrap(), String::from(TITLE))
        );
    }

    #[test]
    fn test_show_watchlist() {
        let c = Command::from_str(SHOW_WATCH_LIST).unwrap();
        assert_eq!(c, Command::ShowWatchlist);
        let c = Command::from_str(SHOW_WATCH_LIST_SHORT).unwrap();
        assert_eq!(c, Command::ShowWatchlist);
    }

    #[test]
    fn test_help() {
        let c = Command::from_str(HELP).unwrap();
        assert_eq!(c, Command::Help(SimpleCommand::General));
        let c = Command::from_str(HELP_SHORT).unwrap();
        assert_eq!(c, Command::Help(SimpleCommand::General));
    }

    #[test]
    fn test_empty_str() {
        let e = Command::from_str("").unwrap_err();
        assert_eq!(e, ParseCommandError::NoCommand);
    }

    #[test]
    fn test_no_arguments_for_add() {
        let e = Command::from_str(ADD_MOVIE).unwrap_err();
        assert_eq!(e, ParseCommandError::NoArgumentsForAdd);
        let e = Command::from_str(ADD_MOVIE_SHORT).unwrap_err();
        assert_eq!(e, ParseCommandError::NoArgumentsForAdd);
    }

    #[test]
    fn test_no_arguments_for_remove() {
        let e = Command::from_str(REMOVE_MOVIE).unwrap_err();
        assert_eq!(e, ParseCommandError::NoArgumentsForRemove);
        let e = Command::from_str(REMOVE_MOVIE_SHORT).unwrap_err();
        assert_eq!(e, ParseCommandError::NoArgumentsForRemove);
    }

    #[test]
    fn test_not_enough_arguments_for_edit() {
        let e = Command::from_str(EDIT_MOVIE).unwrap_err();
        assert_eq!(e, ParseCommandError::NotEnoughArgumentsForEdit);
        let e = Command::from_str(EDIT_MOVIE_SHORT).unwrap_err();
        assert_eq!(e, ParseCommandError::NotEnoughArgumentsForEdit);

        let e = Command::from_str(format!("{} {}", EDIT_MOVIE, ID).as_str()).unwrap_err();
        assert_eq!(e, ParseCommandError::NotEnoughArgumentsForEdit);
        let e = Command::from_str(format!("{} {}", EDIT_MOVIE_SHORT, ID).as_str()).unwrap_err();
        assert_eq!(e, ParseCommandError::NotEnoughArgumentsForEdit);
    }

    #[test]
    fn test_wrong_arguments_for_edit() {
        let e = Command::from_str(format!("{} {} {}", EDIT_MOVIE, TITLE, ID).as_str()).unwrap_err();
        assert_eq!(e, ParseCommandError::WrongArgumentForEdit);
        let e = Command::from_str(format!("{} {} {}", EDIT_MOVIE_SHORT, TITLE, ID).as_str())
            .unwrap_err();
        assert_eq!(e, ParseCommandError::WrongArgumentForEdit);
    }

    #[test]
    fn test_unknown_command() {
        let e = Command::from_str(UNKNOWN).unwrap_err();
        assert_eq!(e, ParseCommandError::UnknownCommand);
    }
}
