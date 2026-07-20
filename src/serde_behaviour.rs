use crate::send_message;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub(crate) const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct PersistedState {
    pub(crate) schema_version: u32,
    pub(crate) watch_list: std::collections::HashMap<u32, crate::movie_behaviour::WatchListEntry>,
    pub(crate) server_id: discord::model::ServerId,
    pub(crate) custom_prefix: char,
    pub(crate) movie_limit_per_user: u32,
    pub(crate) movie_vote_limit: u32,
    pub(crate) next_movie_id: u32,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            watch_list: std::collections::HashMap::new(),
            server_id: discord::model::ServerId(0),
            custom_prefix: '.',
            movie_limit_per_user: 10,
            movie_vote_limit: 2,
            next_movie_id: 0,
        }
    }
}

impl From<&crate::BotData> for PersistedState {
    fn from(bot_data: &crate::BotData) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            watch_list: bot_data.watch_list.clone(),
            server_id: bot_data.server_id,
            custom_prefix: bot_data.custom_prefix,
            movie_limit_per_user: bot_data.movie_limit_per_user,
            movie_vote_limit: bot_data.movie_vote_limit,
            next_movie_id: bot_data.next_movie_id,
        }
    }
}

#[derive(Debug)]
pub(crate) enum PersistenceError {
    Read {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    Parse {
        path: PathBuf,
        line: usize,
        column: usize,
    },
    UnsupportedSchema {
        path: PathBuf,
        version: u32,
    },
    Serialize,
    CreateTemp {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    WriteTemp {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    FlushTemp {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    SyncTemp {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    Rename {
        path: PathBuf,
        kind: io::ErrorKind,
    },
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, kind } => write!(
                formatter,
                "cannot read persisted state at {} ({kind})",
                path.display()
            ),
            Self::Parse { path, line, column } => write!(
                formatter,
                "cannot parse persisted state JSON at {} (line {line}, column {column})",
                path.display()
            ),
            Self::UnsupportedSchema { path, version } => write!(
                formatter,
                "unsupported schema version {version} in persisted state at {}",
                path.display()
            ),
            Self::Serialize => formatter.write_str("cannot serialize persisted state"),
            Self::CreateTemp { path, kind } => write!(
                formatter,
                "cannot create temporary persisted state file beside {} ({kind})",
                path.display()
            ),
            Self::WriteTemp { path, kind } => write!(
                formatter,
                "cannot write temporary persisted state file for {} ({kind})",
                path.display()
            ),
            Self::FlushTemp { path, kind } => write!(
                formatter,
                "cannot flush temporary persisted state file for {} ({kind})",
                path.display()
            ),
            Self::SyncTemp { path, kind } => write!(
                formatter,
                "cannot sync temporary persisted state file for {} ({kind})",
                path.display()
            ),
            Self::Rename { path, kind } => write!(
                formatter,
                "cannot atomically replace persisted state at {} ({kind})",
                path.display()
            ),
        }
    }
}

impl Error for PersistenceError {}

pub(crate) fn load_persisted_state(path: &Path) -> Result<PersistedState, PersistenceError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(PersistedState::default());
        }
        Err(error) => {
            return Err(PersistenceError::Read {
                path: path.to_path_buf(),
                kind: error.kind(),
            });
        }
    };
    let mut json = String::new();
    if let Err(error) = file.read_to_string(&mut json) {
        return Err(PersistenceError::Read {
            path: path.to_path_buf(),
            kind: error.kind(),
        });
    }
    let state =
        serde_json::from_str::<PersistedState>(&json).map_err(|error| PersistenceError::Parse {
            path: path.to_path_buf(),
            line: error.line(),
            column: error.column(),
        })?;
    if state.schema_version != SCHEMA_VERSION {
        return Err(PersistenceError::UnsupportedSchema {
            path: path.to_path_buf(),
            version: state.schema_version,
        });
    }
    Ok(state)
}

pub(crate) fn save_persisted_state(
    path: &Path,
    state: &PersistedState,
) -> Result<(), PersistenceError> {
    let serialized = serde_json::to_vec_pretty(state).map_err(|_| PersistenceError::Serialize)?;
    save_serialized_state(path, &serialized, false)
}

fn save_serialized_state(
    path: &Path,
    serialized: &[u8],
    fail_before_write: bool,
) -> Result<(), PersistenceError> {
    let temporary_path = temporary_path(path);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .map_err(|error| PersistenceError::CreateTemp {
                path: path.to_path_buf(),
                kind: error.kind(),
            })?;
        if fail_before_write {
            return Err(PersistenceError::WriteTemp {
                path: path.to_path_buf(),
                kind: io::ErrorKind::Other,
            });
        }
        file.write_all(serialized)
            .map_err(|error| PersistenceError::WriteTemp {
                path: path.to_path_buf(),
                kind: error.kind(),
            })?;
        file.flush().map_err(|error| PersistenceError::FlushTemp {
            path: path.to_path_buf(),
            kind: error.kind(),
        })?;
        file.sync_all()
            .map_err(|error| PersistenceError::SyncTemp {
                path: path.to_path_buf(),
                kind: error.kind(),
            })?;

        fs::rename(&temporary_path, path).map_err(|error| PersistenceError::Rename {
            path: path.to_path_buf(),
            kind: error.kind(),
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn temporary_path(path: &Path) -> PathBuf {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state");
    directory.join(format!(".{name}.{}.tmp", std::process::id()))
}

pub(crate) fn store_bot_data(bot_data: &crate::BotData) -> Result<(), PersistenceError> {
    let state = PersistedState::from(bot_data);
    save_persisted_state(crate::config::get().data_file(), &state)?;
    send_message::data_saved_successfully(bot_data);
    Ok(())
}

pub(crate) fn store_bot_data_silently(bot_data: &crate::BotData) -> Result<(), PersistenceError> {
    let state = PersistedState::from(bot_data);
    save_persisted_state(crate::config::get().data_file(), &state)
}

#[cfg(test)]
fn save_persisted_state_with_failure_for_test(
    path: &Path,
    state: &PersistedState,
) -> Result<(), PersistenceError> {
    let serialized = serde_json::to_vec_pretty(state).map_err(|_| PersistenceError::Serialize)?;
    save_serialized_state(path, &serialized, true)
}

#[cfg(test)]
mod tests {
    use super::{
        PersistedState, SCHEMA_VERSION, load_persisted_state, save_persisted_state,
        save_persisted_state_with_failure_for_test,
    };
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temporary_data_file(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("discord-movie-night-{test_name}-{unique}.json"))
    }

    #[test]
    fn persists_a_versioned_state_round_trip() {
        let path = temporary_data_file("round-trip");
        let state = PersistedState::default();
        save_persisted_state(&path, &state).expect("state must save");
        let stored = fs::read_to_string(&path).expect("saved state must be readable");
        let loaded = load_persisted_state(&path).expect("saved state must load");
        assert!(stored.contains(&format!("\"schema_version\": {SCHEMA_VERSION}")));
        for transient_field in [
            "bot",
            "tmdb",
            "wait_for_reaction",
            "votes",
            "bot_user",
            "message",
            "server_roles",
            "adding_movie",
        ] {
            assert!(!stored.contains(&format!("\"{transient_field}\"")));
        }
        assert_eq!(loaded.schema_version, SCHEMA_VERSION);
        assert_eq!(loaded.custom_prefix, state.custom_prefix);
        fs::remove_file(path).expect("temporary file must be removed");
    }

    #[test]
    fn invalid_persisted_startup_returns_before_any_stdin_recovery() {
        let path = temporary_data_file("corrupt");
        fs::write(&path, "{ definitely not json").expect("corrupt fixture must be written");
        let error = load_persisted_state(&path)
            .expect_err("corrupt data must fail")
            .to_string();
        assert!(error.contains(&path.display().to_string()));
        assert!(error.contains("parse"));
        fs::remove_file(path).expect("temporary file must be removed");
    }

    #[test]
    fn rejects_an_unknown_schema_version() {
        let path = temporary_data_file("unknown-version");
        fs::write(&path, r#"{"schema_version":999,"watch_list":{},"server_id":0,"custom_prefix":".","movie_limit_per_user":10,"movie_vote_limit":2,"next_movie_id":0}"#).expect("fixture must be written");
        let error = load_persisted_state(&path)
            .expect_err("unknown schema must fail")
            .to_string();
        assert!(error.contains("unsupported schema version"));
        assert!(error.contains("999"));
        fs::remove_file(path).expect("temporary file must be removed");
    }

    #[test]
    fn failed_save_preserves_the_previous_valid_file() {
        let path = temporary_data_file("failed-save");
        let previous = r#"{"schema_version":1,"watch_list":{},"server_id":0,"custom_prefix":".","movie_limit_per_user":10,"movie_vote_limit":2,"next_movie_id":0}"#;
        fs::write(&path, previous).expect("previous state must be written");
        let result = save_persisted_state_with_failure_for_test(&path, &PersistedState::default());
        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(&path).expect("previous state must remain"),
            previous
        );
        fs::remove_file(path).expect("temporary file must be removed");
    }

    #[test]
    fn successful_atomic_save_replaces_existing_state() {
        let path = temporary_data_file("atomic-replace");
        fs::write(&path, "old state").expect("previous state must be written");
        let mut state = PersistedState::default();
        state.next_movie_id = 42;
        save_persisted_state(&path, &state).expect("state must save atomically");
        assert_eq!(
            load_persisted_state(&path)
                .expect("replacement state must load")
                .next_movie_id,
            42
        );
        fs::remove_file(path).expect("temporary file must be removed");
    }
}
