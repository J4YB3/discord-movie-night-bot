use crate::send_message;
use serde_json;
use std::io::Write;

#[derive(Debug)]
pub enum StoreBotDataError {
    Serialization,
    OpenFile,
    Write,
}

/**
 * Tries to store the bot data and preserves the existing command feedback.
 */
pub fn store_bot_data(bot_data: &crate::BotData) -> Result<(), StoreBotDataError> {
    store_bot_data_with_feedback(bot_data, true)
}

/**
 * Tries to store the bot data without sending a user-facing Discord message.
 */
pub fn store_bot_data_silently(bot_data: &crate::BotData) -> Result<(), StoreBotDataError> {
    store_bot_data_with_feedback(bot_data, false)
}

fn store_bot_data_with_feedback(
    bot_data: &crate::BotData,
    send_feedback: bool,
) -> Result<(), StoreBotDataError> {
    tracing::info!(
        event = "persistence_started",
        "bot data persistence started"
    );
    let serialized_bot_data = match serde_json::to_string_pretty(bot_data) {
        Ok(serialized_bot_data) => serialized_bot_data,
        Err(error) => {
            if send_feedback {
                send_message::read_store_data_error(bot_data, error);
            }
            tracing::warn!(
                event = "persistence_failed",
                stage = "serialization",
                "bot data persistence failed"
            );
            return Err(StoreBotDataError::Serialization);
        }
    };

    let file = match open_data_file(true) {
        Ok(file) => file,
        Err(error) => {
            if send_feedback {
                send_message::open_file_error(bot_data, error);
            }
            tracing::warn!(
                event = "persistence_failed",
                stage = "open",
                "bot data persistence failed"
            );
            return Err(StoreBotDataError::OpenFile);
        }
    };

    match write_serialized_bot_data(file, &serialized_bot_data) {
        Ok(()) => {
            if send_feedback {
                send_message::data_saved_successfully(bot_data);
            }
            tracing::info!(
                event = "persistence_succeeded",
                "bot data persistence succeeded"
            );
            Ok(())
        }
        Err(error) => {
            if send_feedback {
                send_message::write_error(bot_data, error);
            }
            tracing::warn!(
                event = "persistence_failed",
                stage = "write",
                "bot data persistence failed"
            );
            Err(StoreBotDataError::Write)
        }
    }
}

fn write_serialized_bot_data(
    mut writer: impl Write,
    serialized_bot_data: &str,
) -> std::io::Result<()> {
    writer.write_all(serialized_bot_data.as_bytes())
}

/**
 * Tries to read the bot data from the file. If something goes wrong it formats the error and returns it inside Err.
 * Otherwise it returns the Ok value containing the created bot data struct
 */
pub fn read_bot_data() -> Result<crate::BotData, String> {
    match open_data_file(false) {
        Ok(mut file) => {
            let mut result_string = String::new();

            use std::io::Read;

            match file.read_to_string(&mut result_string) {
                Ok(_) => match serde_json::from_str::<crate::BotData>(result_string.as_str()) {
                    Ok(bot_data) => Ok(bot_data),
                    Err(error) => Err(format!("{:#?}", error)),
                },
                Err(error) => Err(format!("{:#?}", error)),
            }
        }
        Err(error) => Err(format!("{:#?}", error)),
    }
}

/**
 * Opens the data file where the bot data is stored
 */
fn open_data_file(truncate: bool) -> Result<std::fs::File, std::io::Error> {
    use std::fs::OpenOptions;

    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(truncate)
        .open("discord_movie_night_bot_data.json")
}

#[cfg(test)]
mod tests {
    use super::write_serialized_bot_data;
    use std::io::{self, Write};

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn write_serialized_bot_data_returns_write_failures() {
        assert!(write_serialized_bot_data(FailingWriter, "{}").is_err());
    }
}
