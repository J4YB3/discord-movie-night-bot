use serde_json;
use crate::send_message;

/**
 * Tries to store the bot data. Sends an error message if it failed. Otherwise the file is written
 */
pub fn store_bot_data(bot_data: &crate::BotData) {
    // Try to serialize the bot_data
    let serialize_result = serde_json::to_string_pretty(bot_data);

    if let Err(error) = serialize_result {
        send_message::read_store_data_error(bot_data, error);
        return;
    }

    // Here serialize_result must be valid, so unwrap it
    let serialized_bot_data = serialize_result.unwrap();

    match open_data_file(true) {
        Ok(mut file) => {
            use std::io::Write;

            if let Err(error) = file.write_all(serialized_bot_data.as_bytes()) {
                send_message::write_error(bot_data, error);
            } else {
                send_message::data_saved_successfully(bot_data);
            }
        },
        Err(error) => send_message::open_file_error(bot_data, error)
    }
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
                Ok(_) => {
                    match serde_json::from_str::<crate::BotData>(result_string.as_str()) {
                        Ok(bot_data) => Ok(bot_data),
                        Err(error) => Err(format!("{:#?}", error))
                    }
                },
                Err(error) => Err(format!("{:#?}", error)),
            }
        },
        Err(error) => Err(format!("{:#?}", error))
    }
}

/**
 * Opens the data file where the bot data is stored
 */
fn open_data_file(truncate: bool) -> Result<std::fs::File, std::io::Error> {
    use std::fs::OpenOptions;

    OpenOptions::new().read(true).write(true).create(true).truncate(truncate).open("discord_movie_night_bot_data.json")
}