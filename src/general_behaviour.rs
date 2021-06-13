use crate::COLOR_INFORMATION;
use regex::Regex;

#[derive(Clone, Debug)]
pub enum WaitingForReaction {
    AddMovie(discord::model::MessageId, crate::movie_behaviour::WatchListEntry),
    Vote(discord::model::MessageId),
    AddMovieToWatched(discord::model::MessageId, crate::movie_behaviour::Movie),
}

/**
 * Takes a timestamp from the chrono package and converts it to german date format,
 * translating the english weekday to german in the process.
 */
pub fn timestamp_to_string(timestamp: &chrono::DateTime<chrono::FixedOffset>, include_weekday: bool) -> String {
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
pub fn parse_tmdb_release_date(tmdb_date: String) -> Result<chrono::DateTime<chrono::FixedOffset>, String> {
    let date_with_utc = tmdb_date.clone() + " 12:00:00.000 +0000";
    if let Ok(datetime) = chrono::DateTime::parse_from_str(date_with_utc.as_str(), "%Y-%m-%d %H:%M:%S%.3f %z") {
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
    if let Some(regex_match) = Regex::new(r"[a-z][a-z][0-9]+").unwrap().find(hyperlink.as_str()) {
        Some(regex_match.as_str().to_string())
    } else {
        None
    }
}

/**
 * Takes the budget of a movie and formats it to easy read format (169 mio.)
 */
pub fn format_budget(budget: u64) -> String {
    let budget_string = format!("{}", budget).to_string();
    // 169000000 = 169 mio = 169.000.000

    if budget == 0 {
        return String::from("Unbekannt");
    } else if budget_string.len() <= 3 {
        return budget_string.clone();
    } else {
        let (first, _) = budget_string.split_at(3);
        return match budget_string.len() {
            4..=6 => String::from(format!("{} k", first).as_str()),
            7..=9 => String::from(format!("{} mio.", first).as_str()),
            10..=12 => String::from(format!("{} mrd.", first).as_str()),
            _ => budget_string.clone()
        };
    }
}

/**
 * Returns true if the given ReactionEmoji enum equals the unicode emoji
 */
pub fn reaction_emoji_equals(reaction_emoji: &discord::model::ReactionEmoji, unicode: String) -> bool {
    reaction_emojis_equal(reaction_emoji, &discord::model::ReactionEmoji::Unicode(unicode))
}

/**
 * Returns true if two ReactionEmoji enum values are equal
 */
pub fn reaction_emojis_equal(first: &discord::model::ReactionEmoji, second: &discord::model::ReactionEmoji) -> bool {
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
            if let discord::model::ReactionEmoji::Custom{name: _, id: first_id} = first {
                if let discord::model::ReactionEmoji::Custom{name: _, id: second_id} = second {
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

/**
 * Sets a new custom prefix for all commands
 */
pub fn set_new_prefix(bot_data: &mut crate::BotData, new_prefix: char) {
    let message = bot_data.message.as_ref().expect("Passing message to set_new_prefix function failed.");

    if is_user_administrator(bot_data, message.author.id) {
        bot_data.custom_prefix = new_prefix;

        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed.title(":information_source: Neuer Präfix").description(
                format!(
                    "Der Präfix für alle Kommandos wurde zu `{}` geändert.
                    Bitte benutze nur noch diesen Präfix um auf den Bot zuzugreifen. Der zuvor genutzte Präfix ist nun nicht mehr verfügbar.", 
                    new_prefix
                ).as_str()
            ).color(COLOR_INFORMATION)
        );
    } else {
        let _ = bot_data.bot.send_embed(
            message.channel_id,
            "",
            |embed| embed.title(":information_source: Rechte nicht ausreichend").description(
                format!(
                    "Du benötigst Administrator-Rechte um den Präfix für diesen Bot zu ändern."
                ).as_str()
            ).color(COLOR_INFORMATION)
        );
    }
}

/**
 * Checks all roles of the user for admin permissions and returns true if the user has at least one
 * role with those permissions
 */
pub fn is_user_administrator(bot_data: &crate::BotData, user_id: discord::model::UserId) -> bool {
    let author_role_ids = bot_data.bot.get_member(bot_data.server_id, user_id).expect("Retrieval of author user failed.").roles;

    for role in &bot_data.server_roles {
        if author_role_ids.contains(&role.id) {
            if is_role_administrator(role) {
                return true
            }
        }
    }

    return false
}

/**
 * Returns true if the given role has administrator permissions
 */
fn is_role_administrator(role: &discord::model::Role) -> bool {
    let admin_permissions = discord::model::permissions::Permissions::ADMINISTRATOR;
    role.permissions.contains(admin_permissions)
}