use crate::send_message;
use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub(crate) const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersistedState {
    pub(crate) schema_version: u32,
    pub(crate) watch_list: std::collections::HashMap<u32, crate::movie_behaviour::WatchListEntry>,
    pub(crate) server_id: discord::model::ServerId,
    pub(crate) custom_prefix: char,
    pub(crate) movie_limit_per_user: u32,
    pub(crate) movie_vote_limit: u32,
    pub(crate) next_movie_id: u32,
}

#[derive(Deserialize)]
struct LegacyPersistedState {
    watch_list: std::collections::HashMap<u32, crate::movie_behaviour::WatchListEntry>,
    server_id: discord::model::ServerId,
    custom_prefix: char,
    movie_limit_per_user: u32,
    movie_vote_limit: u32,
    next_movie_id: u32,
}

impl From<LegacyPersistedState> for PersistedState {
    fn from(state: LegacyPersistedState) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            watch_list: state.watch_list,
            server_id: state.server_id,
            custom_prefix: state.custom_prefix,
            movie_limit_per_user: state.movie_limit_per_user,
            movie_vote_limit: state.movie_vote_limit,
            next_movie_id: state.next_movie_id,
        }
    }
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
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        line: usize,
        column: usize,
        source: serde_json::Error,
    },
    UnsupportedSchema {
        path: PathBuf,
        version: u32,
    },
    Serialize {
        source: serde_json::Error,
    },
    CreateTemp {
        path: PathBuf,
        source: io::Error,
    },
    WriteTemp {
        path: PathBuf,
        source: io::Error,
    },
    FlushTemp {
        path: PathBuf,
        source: io::Error,
    },
    SyncTemp {
        path: PathBuf,
        source: io::Error,
    },
    Commit {
        path: PathBuf,
        source: io::Error,
    },
}

impl PersistenceError {
    pub(crate) fn stage(&self) -> &'static str {
        match self {
            Self::Read { .. } => "read",
            Self::Parse { .. } => "parse",
            Self::UnsupportedSchema { .. } => "schema_validation",
            Self::Serialize { .. } => "serialize",
            Self::CreateTemp { .. } => "create_temp",
            Self::WriteTemp { .. } => "write_temp",
            Self::FlushTemp { .. } => "flush_temp",
            Self::SyncTemp { .. } => "sync_temp",
            Self::Commit { .. } => "commit",
        }
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Self::Read { path, .. }
            | Self::Parse { path, .. }
            | Self::UnsupportedSchema { path, .. }
            | Self::CreateTemp { path, .. }
            | Self::WriteTemp { path, .. }
            | Self::FlushTemp { path, .. }
            | Self::SyncTemp { path, .. }
            | Self::Commit { path, .. } => Some(path),
            Self::Serialize { .. } => None,
        }
    }

    pub(crate) fn io_kind(&self) -> Option<io::ErrorKind> {
        match self {
            Self::Read { source, .. }
            | Self::CreateTemp { source, .. }
            | Self::WriteTemp { source, .. }
            | Self::FlushTemp { source, .. }
            | Self::SyncTemp { source, .. }
            | Self::Commit { source, .. } => Some(source.kind()),
            Self::Parse { .. } | Self::UnsupportedSchema { .. } | Self::Serialize { .. } => None,
        }
    }
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(
                formatter,
                "cannot read persisted state at {} ({})",
                path.display(),
                source.kind()
            ),
            Self::Parse {
                path, line, column, ..
            } => write!(
                formatter,
                "cannot parse persisted state JSON at {} (line {line}, column {column})",
                path.display()
            ),
            Self::UnsupportedSchema { path, version } => write!(
                formatter,
                "unsupported schema version {version} in persisted state at {}",
                path.display()
            ),
            Self::Serialize { .. } => formatter.write_str("cannot serialize persisted state"),
            Self::CreateTemp { path, source } => write!(
                formatter,
                "cannot create temporary persisted state file beside {} ({})",
                path.display(),
                source.kind()
            ),
            Self::WriteTemp { path, source } => write!(
                formatter,
                "cannot write temporary persisted state file for {} ({})",
                path.display(),
                source.kind()
            ),
            Self::FlushTemp { path, source } => write!(
                formatter,
                "cannot flush temporary persisted state file for {} ({})",
                path.display(),
                source.kind()
            ),
            Self::SyncTemp { path, source } => write!(
                formatter,
                "cannot sync temporary persisted state file for {} ({})",
                path.display(),
                source.kind()
            ),
            Self::Commit { path, source } => write!(
                formatter,
                "cannot atomically replace persisted state at {} ({})",
                path.display(),
                source.kind()
            ),
        }
    }
}

impl Error for PersistenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. }
            | Self::CreateTemp { source, .. }
            | Self::WriteTemp { source, .. }
            | Self::FlushTemp { source, .. }
            | Self::SyncTemp { source, .. }
            | Self::Commit { source, .. } => Some(source),
            Self::Parse { source, .. } | Self::Serialize { source } => Some(source),
            Self::UnsupportedSchema { .. } => None,
        }
    }
}

pub(crate) fn load_persisted_state(path: &Path) -> Result<PersistedState, PersistenceError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(PersistedState::default());
        }
        Err(error) => {
            return Err(PersistenceError::Read {
                path: path.to_path_buf(),
                source: error,
            });
        }
    };
    let mut json = String::new();
    if let Err(error) = file.read_to_string(&mut json) {
        return Err(PersistenceError::Read {
            path: path.to_path_buf(),
            source: error,
        });
    }
    let value = serde_json::from_str::<serde_json::Value>(&json)
        .map_err(|source| parse_error(path, source))?;
    let state = if value
        .as_object()
        .and_then(|object| object.get("schema_version"))
        .is_some()
    {
        serde_json::from_value::<PersistedState>(value)
            .map_err(|source| parse_error(path, source))?
    } else {
        serde_json::from_value::<LegacyPersistedState>(value)
            .map(PersistedState::from)
            .map_err(|source| parse_error(path, source))?
    };
    if state.schema_version != SCHEMA_VERSION {
        return Err(PersistenceError::UnsupportedSchema {
            path: path.to_path_buf(),
            version: state.schema_version,
        });
    }
    Ok(state)
}

fn parse_error(path: &Path, source: serde_json::Error) -> PersistenceError {
    PersistenceError::Parse {
        path: path.to_path_buf(),
        line: source.line(),
        column: source.column(),
        source,
    }
}

pub(crate) fn save_persisted_state(
    path: &Path,
    state: &PersistedState,
) -> Result<(), PersistenceError> {
    let serialized = serde_json::to_vec_pretty(state)
        .map_err(|source| PersistenceError::Serialize { source })?;
    save_serialized_state(path, &serialized, false)
}

fn save_serialized_state(
    path: &Path,
    serialized: &[u8],
    fail_before_write: bool,
) -> Result<(), PersistenceError> {
    let mut file = AtomicWriteFile::open(path).map_err(|source| PersistenceError::CreateTemp {
        path: path.to_path_buf(),
        source,
    })?;
    if fail_before_write {
        let _ = file.discard();
        return Err(PersistenceError::WriteTemp {
            path: path.to_path_buf(),
            source: io::Error::other("injected write failure"),
        });
    }
    file.write_all(serialized)
        .map_err(|source| PersistenceError::WriteTemp {
            path: path.to_path_buf(),
            source,
        })?;
    file.flush().map_err(|source| PersistenceError::FlushTemp {
        path: path.to_path_buf(),
        source,
    })?;
    file.sync_all()
        .map_err(|source| PersistenceError::SyncTemp {
            path: path.to_path_buf(),
            source,
        })?;
    file.commit().map_err(|source| PersistenceError::Commit {
        path: path.to_path_buf(),
        source,
    })
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
pub(crate) fn save_persisted_state_with_failure_for_test(
    path: &Path,
    state: &PersistedState,
) -> Result<(), PersistenceError> {
    let serialized = serde_json::to_vec_pretty(state)
        .map_err(|source| PersistenceError::Serialize { source })?;
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
    fn migrates_pr64_legacy_state_and_ignores_runtime_fields() {
        let path = temporary_data_file("pr64-legacy");
        let legacy = include_str!("fixtures/pr64-state.json");
        let legacy_value: serde_json::Value =
            serde_json::from_str(legacy).expect("legacy fixture must be valid JSON");
        let legacy_fields = legacy_value
            .as_object()
            .expect("legacy fixture must be a JSON object");
        for field in [
            "watch_list",
            "votes",
            "bot_user",
            "message",
            "server_roles",
            "server_id",
            "custom_prefix",
            "movie_limit_per_user",
            "movie_vote_limit",
            "next_movie_id",
        ] {
            assert!(legacy_fields.contains_key(field));
        }
        for skipped_field in ["schema_version", "wait_for_reaction", "adding_movie"] {
            assert!(!legacy_fields.contains_key(skipped_field));
        }
        fs::write(&path, legacy).expect("legacy fixture must be written");

        let state = load_persisted_state(&path).expect("legacy state must migrate");

        assert_eq!(state.schema_version, SCHEMA_VERSION);
        assert!(state.watch_list.is_empty());
        assert_eq!(state.server_id.0, 123_456_789);
        assert_eq!(state.custom_prefix, '!');
        assert_eq!(state.movie_limit_per_user, 7);
        assert_eq!(state.movie_vote_limit, 3);
        assert_eq!(state.next_movie_id, 42);

        save_persisted_state(&path, &state).expect("migrated state must save");
        let saved = fs::read_to_string(&path).expect("migrated state must be readable");
        assert!(saved.contains(&format!("\"schema_version\": {SCHEMA_VERSION}")));
        for runtime_field in [
            "votes",
            "bot_user",
            "message",
            "server_roles",
            "wait_for_reaction",
            "adding_movie",
        ] {
            assert!(!saved.contains(&format!("\"{runtime_field}\"")));
        }
        fs::remove_file(path).expect("temporary file must be removed");
    }

    #[test]
    fn versioned_state_with_legacy_runtime_fields_is_not_migrated() {
        let path = temporary_data_file("versioned-runtime-fields");
        fs::write(
            &path,
            r#"{"schema_version":1,"watch_list":{},"server_id":0,"custom_prefix":".","movie_limit_per_user":10,"movie_vote_limit":2,"next_movie_id":0,"votes":{}}"#,
        )
        .expect("versioned fixture must be written");

        assert!(load_persisted_state(&path).is_err());
        fs::remove_file(path).expect("temporary file must be removed");
    }

    #[test]
    fn invalid_persisted_state_reports_a_parse_error() {
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
    fn failed_save_preserves_the_previous_valid_file_and_cleans_up_temps() {
        let directory = temporary_data_file("failed-save").with_extension("directory");
        fs::create_dir(&directory).expect("test directory must be created");
        let path = directory.join("state.json");
        let previous = r#"{"schema_version":1,"watch_list":{},"server_id":0,"custom_prefix":".","movie_limit_per_user":10,"movie_vote_limit":2,"next_movie_id":0}"#;
        fs::write(&path, previous).expect("previous state must be written");
        let result = save_persisted_state_with_failure_for_test(&path, &PersistedState::default());
        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(&path).expect("previous state must remain"),
            previous
        );
        let entries = fs::read_dir(&directory)
            .expect("test directory must be readable")
            .map(|entry| {
                entry
                    .expect("test directory entry must be readable")
                    .file_name()
            })
            .collect::<Vec<_>>();
        assert_eq!(entries, vec![std::ffi::OsString::from("state.json")]);
        fs::remove_dir_all(directory).expect("test directory must be removed");
    }

    #[test]
    fn successful_atomic_save_replaces_existing_state() {
        let path = temporary_data_file("atomic-replace");
        fs::write(&path, "old state").expect("previous state must be written");
        let state = PersistedState {
            next_movie_id: 42,
            ..PersistedState::default()
        };
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
