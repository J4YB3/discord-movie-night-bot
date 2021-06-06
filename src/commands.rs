use std::{str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    AddMovie(String),
    RemoveMovieByTitle(String),
    RemoveMovieById(u32),
    ShowWatchlist(String),
    Help(SimpleCommand),
    Prefix(char),
    SetMovieLimit(u32),
    ShowMovieLimit,
    History(String),
    SetStatus(u32, String),
    Unavailable(u32),
    Watched(u32, String),
    ShowMovieByTitle(String),
    ShowMovieById(u32),
    SearchMovie(String),
    CreateVote(String, Vec<String>),
    SendVote,
    SendVoteWithUserId(u64),
    CloseVote,
    SetMovieVoteLimit(u32),
    ShowMovieVoteLimit,
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
    NoArgumentsForShowMovie,
    NoArgumentsForSearchMovie,
    NoArgumentsForCreateVote,
    WrongArgumentsForMovieLimit,
    WrongArgumentsForMovieVoteLimit,
    WrongArgumentsForSendVoteWithUserId,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimpleCommand {
    General,
    Quit,
    Add,
    Remove,
    ShowWatchlist,
    Help,
    Prefix,
    MovieLimit,
    History,
    Status,
    Unavailable,
    Watched,
    ShowMovie,
    Search,
    CreateVote,
    SendVote,
    CloseVote,
    MovieVoteLimit,
    Unknown(String),
}

impl From<&str> for SimpleCommand {
    fn from(s: &str) -> Self {
        match s {
            "" => Self::General,
            QUIT => Self::Quit,
            ADD_MOVIE | ADD_MOVIE_SHORT => Self::Add,
            REMOVE_MOVIE | REMOVE_MOVIE_SHORT => Self::Remove,
            SHOW_WATCH_LIST | SHOW_WATCH_LIST_SHORT => Self::ShowWatchlist,
            HELP | HELP_SHORT => Self::Help,
            PREFIX => Self::Prefix,
            SHOW_HISTORY | SHOW_HISTORY_SHORT => Self::History,
            SET_STATUS | SET_STATUS_SHORT => Self::Status,
            SET_STATUS_UNAVAILABLE | SET_STATUS_UNAVAILABLE_SHORT => Self::Unavailable,
            SET_STATUS_WATCHED | SET_STATUS_WATCHED_SHORT => Self::Watched,
            SHOW_MOVIE | SHOW_MOVIE_SHORT => Self::ShowMovie,
            SEARCH_MOVIE | SEARCH_MOVIE_SHORT => Self::Search,
            CREATE_VOTE | CREATE_VOTE_SHORT => Self::CreateVote,
            SEND_VOTE | SEND_VOTE_SHORT => Self::SendVote,
            CLOSE_VOTE | CLOSE_VOTE_SHORT => Self::CloseVote,
            MOVIE_LIMIT | MOVIE_LIMIT_SHORT => Self::MovieLimit,
            MOVIE_VOTE_LIMIT | MOVIE_VOTE_LIMIT_SHORT => Self::MovieVoteLimit,
            st => Self::Unknown(String::from(st)),
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
            },
            SHOW_MOVIE | SHOW_MOVIE_SHORT => {
                // First argument should be a title consisting of multiple words, therefore join them with spaces
                // In case it is an ID nothing will happen
                let argument = arguments.join(" ");
                if argument.is_empty() {
                    return Err(ParseCommandError::NoArgumentsForShowMovie);
                }

                // Try to parse the first argument to u32. If that is not possible assume it's a title instead of an id.
                if let Ok(n) = argument.parse::<u32>() {
                    Self::ShowMovieById(n)
                } else {
                    Self::ShowMovieByTitle(argument)
                }
            },
            SEARCH_MOVIE | SEARCH_MOVIE_SHORT => {
                let title = arguments.join(" ");
                if title.is_empty() {
                    return Err(ParseCommandError::NoArgumentsForSearchMovie); // Returning bypasses Ok-wrapping.
                }

                Self::SearchMovie(title)
            },
            CREATE_VOTE | CREATE_VOTE_SHORT => {
                let mut vote_parameters: Vec<String> 
                    = arguments
                        .join(" ") // Joins the (possibly more than one) arguments to conserve spaces
                        .split("|") // Split the resulting string at the pipes
                        .map(|x| x.trim().to_string()) // Remove the leading and trailing white spaces of every option
                        .collect();

                if vote_parameters.len() == 0 {
                    return Err(ParseCommandError::NoArgumentsForCreateVote);
                }

                let vote_title = vote_parameters.remove(0);

                Self::CreateVote(vote_title, vote_parameters)
            },
            SEND_VOTE | SEND_VOTE_SHORT => {
                let argument = arguments.join(" ");

                if argument.is_empty() {
                    return Ok(Self::SendVote);
                }

                if argument.contains("!") {
                    if let Some(remainder) = argument.strip_prefix("<@!") {
                        if let Some(remainder) = remainder.strip_suffix(">") {
                            if let Ok(user_id) = remainder.parse::<u64>() {
                                return Ok(Command::SendVoteWithUserId(user_id));
                            }
                        }
                    }

                    return Err(ParseCommandError::WrongArgumentsForSendVoteWithUserId);
                } else {
                    if let Some(remainder) = argument.strip_prefix("<@") {
                        if let Some(remainder) = remainder.strip_suffix(">") {
                            if let Ok(user_id) = remainder.parse::<u64>() {
                                return Ok(Command::SendVoteWithUserId(user_id));
                            }
                        }
                    }

                    return Err(ParseCommandError::WrongArgumentsForSendVoteWithUserId);
                }
            },
            CLOSE_VOTE | CLOSE_VOTE_SHORT => Self::CloseVote,
            MOVIE_LIMIT | MOVIE_LIMIT_SHORT => {
                let argument = arguments.join(" ");
                if argument.is_empty() {
                    return Ok(Self::ShowMovieLimit);
                }

                // Try to parse the first argument to u32. If that is not possible assume it's a title instead of an id.
                if let Ok(n) = argument.parse::<u32>() {
                    Self::SetMovieLimit(n)
                } else {
                    return Err(ParseCommandError::WrongArgumentsForMovieLimit);
                }
            },
            MOVIE_VOTE_LIMIT | MOVIE_VOTE_LIMIT_SHORT => {
                let argument = arguments.join(" ");
                if argument.is_empty() {
                    return Ok(Self::ShowMovieVoteLimit);
                }

                // Try to parse the first argument to u32. If that is not possible assume it's a title instead of an id.
                if let Ok(n) = argument.parse::<u32>() {
                    Self::SetMovieVoteLimit(n)
                } else {
                    return Err(ParseCommandError::WrongArgumentsForMovieVoteLimit);
                }
            },
            _ => return Err(ParseCommandError::UnknownCommand),
        })
    }
}

// Command, Usage | Description
pub const QUIT: &str = "quit"; // !quit | Quits the bot and saves all changes
pub const ADD_MOVIE: &str = "add_movie"; // !add_movie <title|imdb_link> | Adds a movie to the watch list
pub const ADD_MOVIE_SHORT: &str = "am"; // !am <title|imdb_link> | Short form for add_movie
pub const REMOVE_MOVIE: &str = "remove_movie"; // !remove_movie <title|id> | Removes a movie by id or by title from the watch list
pub const REMOVE_MOVIE_SHORT: &str = "rm"; // !rm <title|id> | Short form for remove_movie
pub const SHOW_WATCH_LIST: &str = "watch_list"; // !watch_list <optional: order> | Shows the full watch list
pub const SHOW_WATCH_LIST_SHORT: &str = "wl"; // !wl <optional: order> | Short form for watch_list
pub const HELP: &str = "help"; // !help <optional: command> | Shows a list of available commands or help to one specific command
pub const HELP_SHORT: &str = "h"; // !h <optional: command> | Short form for help
pub const PREFIX: &str = "prefix"; // !prefix <char> | Sets a custom prefix. Must be a single character
pub const MOVIE_LIMIT: &str = "movie_limit"; // !movie_limit <optional: number> | Sets the maximum amount of movies each user can add
pub const MOVIE_LIMIT_SHORT: &str = "ml"; // !ml <optional: number> | Short form for movie_limit
pub const SHOW_HISTORY: &str = "history"; // !history <optional: order> | Shows a list of all movies that have been watched already or that have the status 'removed'
pub const SHOW_HISTORY_SHORT: &str = "hs"; // !h <optional: order> | Short form for history
pub const SET_STATUS: &str = "set_status"; // !set_status <id> <movie_status> | Sets the status of a movie
pub const SET_STATUS_SHORT: &str = "st"; // !st <id> <movie_status> | Short form for set_status
pub const SET_STATUS_UNAVAILABLE: &str = "unavailable"; // !unavailable <id> | Sets the given movie with id to the unavailable status
pub const SET_STATUS_UNAVAILABLE_SHORT: &str = "un"; // !un <id> | Short form for unavailable
pub const SET_STATUS_WATCHED: &str = "watched"; // !watched <id> <optional: date> | Sets the given movie with id to watched. If date is given the timestamp is set to this date
pub const SET_STATUS_WATCHED_SHORT: &str = "wa"; // !wa <id> <optional: date> | Short form for watched
pub const SHOW_MOVIE: &str = "show_movie"; // !show_movie <title|id> | Shows information about a movie in the watch list or the history
pub const SHOW_MOVIE_SHORT: &str = "sm"; // !sm <title|id> | Short form for show_movie
pub const SEARCH_MOVIE: &str = "search_movie"; // !search_movie <title|imdb_link> | Searches TMDb for the given movie and displays its information
pub const SEARCH_MOVIE_SHORT: &str = "search"; // !search <title|imdb_link> | Short form for search_movie

// Voting
pub const CREATE_VOTE: &str = "create_vote"; // !create_vote <title>|<option1>|<option2>|... | Creates a new vote and displays its information
pub const CREATE_VOTE_SHORT: &str = "cv"; // !cv <title>|<option1>|<option2>|... | Short form for create_vote
pub const SEND_VOTE: &str = "send_vote"; // !send_vote <optional: @OtherUser> | Sends the current vote message of the user again
pub const SEND_VOTE_SHORT: &str = "sv"; // !sv <optional: @OtherUser> | Short form for send_vote
pub const CLOSE_VOTE: &str = "close_vote"; // !close_vote | Closes the current vote of the user
pub const CLOSE_VOTE_SHORT: &str = "xv"; // !xv | Short form for close_vote
pub const MOVIE_VOTE_LIMIT: &str = "movie_vote_limit"; // !movie_vote_limit <optional: number> | Sets the amount of movies that are selected for a new movie vote
pub const MOVIE_VOTE_LIMIT_SHORT: &str = "mvl"; // !mvl <optional: number> | Short form for movie_vote_limit