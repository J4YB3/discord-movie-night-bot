use std::{env, error::Error, fmt, path::PathBuf, sync::OnceLock};

const DEFAULT_DATA_FILE: &str = "discord_movie_night_bot_data.json";

static CONFIG: OnceLock<Config> = OnceLock::new();

pub(crate) struct Config {
    discord_token: String,
    tmdb_api_key: String,
    data_file: PathBuf,
    log_filter: String,
}

pub(crate) fn initialize() -> Result<(), ConfigError> {
    CONFIG
        .set(Config::from_env()?)
        .map_err(|_| ConfigError::AlreadyInitialized)
}

pub(crate) fn get() -> &'static Config {
    CONFIG
        .get()
        .expect("configuration must be initialized before use")
}

impl Config {
    fn from_env() -> Result<Self, ConfigError> {
        Self::from_environment(|key| env::var(key).ok())
    }

    fn from_environment(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let discord_token = required_value(&get, "DISCORD_TOKEN");
        let tmdb_api_key = required_value(&get, "TMDB_API_KEY");

        let missing_variables = [
            ("DISCORD_TOKEN", discord_token.is_none()),
            ("TMDB_API_KEY", tmdb_api_key.is_none()),
        ]
        .into_iter()
        .filter_map(|(name, missing)| missing.then_some(name))
        .collect::<Vec<_>>();

        if !missing_variables.is_empty() {
            return Err(ConfigError::MissingRequiredVariables(missing_variables));
        }

        Ok(Self {
            discord_token: discord_token.expect("required configuration was validated"),
            tmdb_api_key: tmdb_api_key.expect("required configuration was validated"),
            data_file: optional_value(&get, "DISCORD_MOVIE_NIGHT_DATA_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_FILE)),
            log_filter: select_log_filter(&get),
        })
    }

    pub(crate) fn discord_token(&self) -> &str {
        &self.discord_token
    }

    pub(crate) fn tmdb_api_key(&self) -> &str {
        &self.tmdb_api_key
    }

    pub(crate) fn data_file(&self) -> &std::path::Path {
        &self.data_file
    }

    pub(crate) fn log_filter(&self) -> &str {
        &self.log_filter
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("discord_token", &"<redacted>")
            .field("tmdb_api_key", &"<redacted>")
            .field("data_file", &self.data_file)
            .field("log_filter", &self.log_filter)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) enum ConfigError {
    MissingRequiredVariables(Vec<&'static str>),
    AlreadyInitialized,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredVariables(variables) => {
                write!(
                    formatter,
                    "missing required environment variables: {}",
                    variables.join(", ")
                )
            }
            Self::AlreadyInitialized => formatter.write_str("configuration is already initialized"),
        }
    }
}

impl Error for ConfigError {}

fn required_value(get: &impl Fn(&str) -> Option<String>, key: &str) -> Option<String> {
    get(key).filter(|value| !value.is_empty())
}

fn optional_value(get: &impl Fn(&str) -> Option<String>, key: &str) -> Option<String> {
    get(key).filter(|value| !value.is_empty())
}

fn select_log_filter(get: impl Fn(&str) -> Option<String>) -> String {
    optional_value(&get, "DISCORD_MOVIE_NIGHT_LOG_FILTER")
        .or_else(|| optional_value(&get, "RUST_LOG"))
        .unwrap_or_else(|| "info".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{Config, select_log_filter};
    use std::collections::BTreeMap;

    #[test]
    fn reports_each_missing_required_variable_without_exposing_present_secrets() {
        let environment = BTreeMap::from([("TMDB_API_KEY", "tmdb-secret")]);

        let error = Config::from_environment(|key| environment.get(key).map(ToString::to_string))
            .expect_err("a missing Discord token must fail configuration loading");
        let error = error.to_string();

        assert!(error.contains("DISCORD_TOKEN"));
        assert!(!error.contains("tmdb-secret"));
    }

    #[test]
    fn debug_redacts_secret_values() {
        let environment = BTreeMap::from([
            ("DISCORD_TOKEN", "discord-secret"),
            ("TMDB_API_KEY", "tmdb-secret"),
        ]);

        let config = Config::from_environment(|key| environment.get(key).map(ToString::to_string))
            .expect("configuration with credentials must load");
        let debug = format!("{config:?}");

        assert!(!debug.contains("discord-secret"));
        assert!(!debug.contains("tmdb-secret"));
    }

    #[test]
    fn log_filter_prefers_the_application_variable() {
        let environment = BTreeMap::from([
            ("DISCORD_MOVIE_NIGHT_LOG_FILTER", "debug"),
            ("RUST_LOG", "warn"),
        ]);

        assert_eq!(
            select_log_filter(|key| environment.get(key).map(ToString::to_string)),
            "debug"
        );
    }

    #[test]
    fn log_filter_uses_rust_log_when_application_variable_is_absent() {
        let environment = BTreeMap::from([("RUST_LOG", "warn")]);

        assert_eq!(
            select_log_filter(|key| environment.get(key).map(ToString::to_string)),
            "warn"
        );
    }

    #[test]
    fn log_filter_defaults_to_info() {
        assert_eq!(select_log_filter(|_| None), "info");
    }
}
