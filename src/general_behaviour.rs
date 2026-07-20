use crate::COLOR_INFORMATION;
use regex::Regex;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WaitingForReaction {
    AddMovie(
        discord::model::Message,
        crate::movie_behaviour::WatchListEntry,
    ),
    Vote(discord::model::Message),
    AddMovieToWatched(discord::model::Message, crate::movie_behaviour::Movie),
    WatchListPagination(
        discord::model::Message,
        crate::movie_behaviour::SortedMovieList,
        /*curr_page:*/ usize,
    ),
    HistoryPagination(
        discord::model::Message,
        crate::movie_behaviour::SortedMovieList,
        /*curr_page:*/ usize,
    ),
}

/**
 * Takes a timestamp from the chrono package and converts it to german date format,
 * translating the english weekday to german in the process.
 */
pub fn timestamp_to_string(
    timestamp: &chrono::DateTime<chrono::FixedOffset>,
    include_weekday: bool,
) -> String {
    let date_format = timestamp.format("%d.%m.%Y");

    if include_weekday {
        let day = match format!("{}", timestamp.format("%A")).as_str() {
            "Monday" => "Montag",
            "Tuesday" => "Dienstag",
            "Wednesday" => "Mittwoch",
            "Thursday" => "Donnerstag",
            "Friday" => "Freitag",
            "Saturday" => "Samstag",
            "Sunday" => "Sonntag",
            _ => "",
        };

        format!("{}, {}", day, date_format).to_string()
    } else {
        format!("{}", date_format).to_string()
    }
}

/**
 * Takes a movie release date in the format yyyy-mm-dd and parses it to the chrono datetime format
 */
pub fn parse_tmdb_release_date(
    tmdb_date: String,
) -> Result<chrono::DateTime<chrono::FixedOffset>, String> {
    let date_with_utc = tmdb_date.clone() + " 12:00:00.000 +0000";
    if let Ok(datetime) =
        chrono::DateTime::parse_from_str(date_with_utc.as_str(), "%Y-%m-%d %H:%M:%S%.3f %z")
    {
        Ok(datetime)
    } else {
        Err("Parsing of movie release date failed".to_string())
    }
}

/**
 * Takes an IMDb link and extracts the IMDb ID
 */
pub fn parse_imdb_link_id(hyperlink: String) -> Option<String> {
    // Example link: https://www.imdb.com/title/tt0816692/?ref_=fn_al_tt_2
    if let Some(regex_match) = Regex::new(r"[a-z][a-z][0-9]+")
        .unwrap()
        .find(hyperlink.as_str())
    {
        Some(regex_match.as_str().to_string())
    } else {
        None
    }
}

/**
 * Takes an TMDb link and extracts the TMDb ID
 */
pub fn parse_tmdb_link_id(hyperlink: String) -> Option<u64> {
    // Example link: https://www.themoviedb.org/movie/9806-the-incredibles
    if let Some(regex_match) = Regex::new(r"/[0-9]*-").unwrap().find(hyperlink.as_str()) {
        let match_str = regex_match.as_str();
        // Remove the slash and the hyphen for return value
        let match_string = match_str[1..match_str.len() - 1].to_string();

        if let Ok(id) = match_string.parse::<u64>() {
            return Some(id);
        }
    }

    None
}

/**
 * Takes the budget of a movie and formats it to easy read format (169 mio.)
 */
pub fn format_budget(budget: u64) -> String {
    match budget {
        0 => "Unbekannt".to_string(),
        1_000..=999_999 => format_budget_unit(budget, 1_000, "k", Some((1_000_000, "mio."))),
        1_000_000..=999_999_999 => {
            format_budget_unit(budget, 1_000_000, "mio.", Some((1_000_000_000, "mrd.")))
        }
        1_000_000_000..=999_999_999_999 => format_budget_unit(budget, 1_000_000_000, "mrd.", None),
        _ => budget.to_string(),
    }
}

fn format_budget_unit(
    budget: u64,
    divisor: u64,
    unit: &str,
    promoted_unit: Option<(u64, &str)>,
) -> String {
    let rounded_tenths = ((budget % divisor) * 10 + divisor / 2) / divisor;
    let whole = budget / divisor + rounded_tenths / 10;
    let decimal = rounded_tenths % 10;

    if whole == 1_000 {
        if let Some((promoted_divisor, promoted_unit)) = promoted_unit {
            return format_budget_unit(budget, promoted_divisor, promoted_unit, None);
        }
    }

    if decimal == 0 {
        format!("{whole} {unit}")
    } else {
        format!("{whole},{decimal} {unit}")
    }
}

/**
 * Returns true if the given ReactionEmoji enum equals the unicode emoji
 */
pub fn reaction_emoji_equals(
    reaction_emoji: &discord::model::ReactionEmoji,
    unicode: String,
) -> bool {
    reaction_emojis_equal(
        reaction_emoji,
        &discord::model::ReactionEmoji::Unicode(unicode),
    )
}

/**
 * Returns true if two ReactionEmoji enum values are equal
 */
pub fn reaction_emojis_equal(
    first: &discord::model::ReactionEmoji,
    second: &discord::model::ReactionEmoji,
) -> bool {
    if let discord::model::ReactionEmoji::Unicode(first_string) = first {
        if let discord::model::ReactionEmoji::Unicode(second_string) = second {
            first_string == second_string
        } else {
            false
        }
    } else {
        if let discord::model::ReactionEmoji::Unicode(_) = second {
            false
        } else {
            if let discord::model::ReactionEmoji::Custom {
                name: _,
                id: first_id,
            } = first
            {
                if let discord::model::ReactionEmoji::Custom {
                    name: _,
                    id: second_id,
                } = second
                {
                    first_id == second_id
                }
                // If the second emoji is none of its two enum options return false
                else {
                    false
                }
            }
            // If the first emoji is none of its two enum options return false
            else {
                false
            }
        }
    }
}

/**
 * Returns the link for the no-image-available image
 */
pub fn get_no_image_available_url() -> &'static str {
    "https://upload.wikimedia.org/wikipedia/commons/thumb/6/65/No-Image-Placeholder.svg/330px-No-Image-Placeholder.svg.png"
}

/**
 * Returns the link to the TMDb logo for attribution
 */
pub fn get_tmdb_attribution_icon_url() -> &'static str {
    "https://www.themoviedb.org/assets/2/v4/logos/312x276-primary-green-74212f6247252a023be0f02a5a45794925c3689117da9d20ffe47742a665c518.png"
}

/// Records a Discord operation failure that is intentionally nonfatal.
///
/// The error value is consumed without logging it because Discord errors can contain
/// response details that should not be emitted to operational logs.
pub fn trace_nonfatal_discord_result<T, E>(result: Result<T, E>, operation: &'static str) {
    if result.is_err() {
        tracing::warn!(
            event = "discord_nonfatal_operation_failed",
            operation,
            "Discord operation failed"
        );
    }
}

/**
 * Sets a new custom prefix for all commands
 */
pub fn set_new_prefix(bot_data: &mut crate::BotData, new_prefix: char) {
    let message = bot_data
        .message
        .as_ref()
        .expect("Passing message to set_new_prefix function failed.");

    if is_user_administrator(bot_data, message.author.id) {
        bot_data.custom_prefix = new_prefix;

        trace_nonfatal_discord_result(bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed.title(":information_source: Neuer Präfix").description(
                format!(
                    "Der Präfix für alle Kommandos wurde zu `{}` geändert.
                    Bitte benutze nur noch diesen Präfix um auf den Bot zuzugreifen. Der zuvor genutzte Präfix ist nun nicht mehr verfügbar.",
                    new_prefix
                ).as_str()
            ).color(COLOR_INFORMATION)
        ), "send_embed");
    } else {
        trace_nonfatal_discord_result(
            bot_data.bot.send_embed(message.channel_id, "", |embed| {
                embed
                    .title(":information_source: Rechte nicht ausreichend")
                    .description(
                        format!(
                        "Du benötigst Administrator-Rechte um den Präfix für diesen Bot zu ändern."
                    )
                        .as_str(),
                    )
                    .color(COLOR_INFORMATION)
            }),
            "send_embed",
        );
    }
}

/**
 * Checks all roles of the user for admin permissions and returns true if the user has at least one
 * role with those permissions
 */
pub fn is_user_administrator(bot_data: &crate::BotData, user_id: discord::model::UserId) -> bool {
    let author_role_ids = match bot_data.bot.get_member(bot_data.server_id, user_id) {
        Ok(member) => member.roles,
        Err(_) => {
            tracing::warn!(
                event = "discord_member_lookup_failed",
                server_id = bot_data.server_id.0,
                user_id = user_id.0,
                "failed to retrieve Discord member"
            );
            return false;
        }
    };

    for role in &bot_data.server_roles {
        if author_role_ids.contains(&role.id) {
            if is_role_administrator(role) {
                return true;
            }
        }
    }

    return false;
}

/**
 * Returns true if the given role has administrator permissions
 */
fn is_role_administrator(role: &discord::model::Role) -> bool {
    let admin_permissions = discord::model::permissions::Permissions::ADMINISTRATOR;
    role.permissions.contains(admin_permissions)
}

/**
 * Removes all reactions on all messages that are stored in waiting_for_reactions
 */
pub fn remove_all_reactions_on_all_waiting_for_reaction_messages(bot_data: &crate::BotData) {
    for waiting in bot_data.wait_for_reaction.iter() {
        match waiting {
            WaitingForReaction::AddMovie(message, _)
            | WaitingForReaction::AddMovieToWatched(message, _) => {
                remove_reactions_on_message(bot_data, &message, vec!["✅", "❎"])
            }
            WaitingForReaction::HistoryPagination(message, _, _)
            | WaitingForReaction::WatchListPagination(message, _, _) => {
                remove_reactions_on_message(bot_data, &message, vec!["⬅️", "➡️"])
            }
            WaitingForReaction::Vote(message) => {
                let vote = bot_data.votes.get(&message.id.0);

                if let Some(vote) = vote {
                    crate::voting_behaviour::remove_all_reactions_on_previous_vote(
                        bot_data,
                        vote,
                        (&message.channel_id, &message.id),
                    );
                }
            }
        }
    }
}

/**
 * Removes the given reactions on a message. Example emojis parameter: vec!["✅", "❎"]
 */
pub fn remove_reactions_on_message(
    bot_data: &crate::BotData,
    message: &discord::model::Message,
    emojis: Vec<&str>,
) {
    for emoji in emojis {
        if bot_data
            .bot
            .delete_reaction(
                message.channel_id,
                message.id,
                None,
                discord::model::ReactionEmoji::Unicode(emoji.to_string()),
            )
            .is_err()
        {
            tracing::warn!(
                operation = "delete_reaction",
                "Discord cleanup operation failed"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{format_budget, trace_nonfatal_discord_result};
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    #[derive(Clone)]
    struct TestWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for TestWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for TestWriter {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn captured_events(run: impl FnOnce()) -> String {
        let output = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_writer(TestWriter(Arc::clone(&output)))
            .finish();

        tracing::subscriber::with_default(subscriber, run);

        String::from_utf8(output.lock().unwrap().clone()).unwrap()
    }

    #[test]
    fn trace_nonfatal_discord_result_emits_a_safe_static_event_for_failures() {
        let events = captured_events(|| {
            trace_nonfatal_discord_result::<(), _>(
                Err("sensitive Discord response details"),
                "test_operation",
            );
        });

        assert!(events.contains("discord_nonfatal_operation_failed"));
        assert!(events.contains("test_operation"));
        assert!(!events.contains("sensitive Discord response details"));
    }

    #[test]
    fn trace_nonfatal_discord_result_emits_no_failure_event_for_successes() {
        let events = captured_events(|| {
            trace_nonfatal_discord_result::<(), &str>(Ok(()), "test_operation");
        });

        assert!(!events.contains("discord_nonfatal_operation_failed"));
    }

    #[test]
    fn format_budget_uses_the_correct_unit_and_precision() {
        assert_eq!(format_budget(999), "999");
        assert_eq!(format_budget(1_500), "1,5 k");
        assert_eq!(format_budget(1_999), "2 k");
        assert_eq!(format_budget(999_999), "1 mio.");
        assert_eq!(format_budget(1_999_999), "2 mio.");
        assert_eq!(format_budget(169_000_000), "169 mio.");
        assert_eq!(format_budget(1_500_000_000), "1,5 mrd.");
        assert_eq!(format_budget(0), "Unbekannt");
    }
}
