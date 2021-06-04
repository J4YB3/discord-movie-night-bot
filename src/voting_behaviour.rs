use rand::distributions::{Distribution, Uniform};
use std::collections::HashSet;

#[derive(Clone)]
pub enum VoteOptionEnum {
    GeneralVoteOption(VoteOption<String>),
    MovieVoteOption(VoteOption<crate::movie_behaviour::Movie>),
}

#[derive(Clone)]
pub struct VoteOption<T> {
    emoji: String,
    cargo: T,
    votes: Vec<discord::model::UserId>,
}

#[derive(Clone)]
pub struct Vote {
    creator: discord::model::User,
    creation_date: chrono::DateTime<chrono::FixedOffset>,
    title: String,
    options: Vec<VoteOptionEnum>,
    message_id: discord::model::MessageId,
}

/**
 * Returns None if the given amount is greater than the amount of stored emojis
 * (which is 150 currently). Otherwise returns a unique set of random emojis,
 * which size is equal to the given amount
 */
fn get_random_unique_emojis(amount: usize) -> Option<HashSet<String>> {
    let emoji_list : Vec<&str> = 
    vec![
        "🐼", "🌵", "🐨", "🐭", "🌹", "🐅", "🌷", "🐜", "🐤", "🦇",
        "🐻", "🐦", "🌼", "🐡", "🐗", "💐", "🐛", "🦋", "🐪", "🐱",
        "🌸", "🐔", "🐵", "🐮", "🐊", "🐶", "🐬", "🐙", "🐲", "🦆",
        "🦅", "🐘", "🌲", "🐑", "🍂", "🐟", "🍀", "🦊", "🐸", "🐣",
        "🦍", "🐹", "🌿", "🌺", "🐝", "🐴", "🐞", "🦁", "🦎", "🍁",
        "🦉", "🐧", "🐷", "🐩", "🐰", "🦏", "🦂", "🐌", "🐍", "🐚", 
        "🐳", "🌻", "🐯", "🐠", "🦃", "🐢", "🐫", "🦄", "🦢", "🐺", 
        "🦚", "🦜", "🦗", "🦌", "🦒", "🦓", "🦔", "🦘", "🦝", "🦈", 
        "🦕", "🦖", "🦧", "🦥", "🦦", "🦨", "🦩", "🍭", "🍏", "🎂", 
        "🥑", "🍳", "🍌", "🍞", "🍒", "🍓", "🍐", "🍕", "🍋", "🥓",
        "🌯", "🥕", "🧀", "🍫", "🍪", "🦀", "🥐", "🥒", "🍩", "🌽",
        "🍟", "🍤", "🍇", "🥗", "🍔", "🌭", "🍨", "🥝", "🍖", "🍄",
        "🥞", "🍑", "🥜", "🍿", "🥔", "🍗", "🍎", "🍙", "🍠", "🍝",
        "🍜", "🍣", "🌮", "🍊", "🍅", "🍉", "🥠", "🍍", "🥥", "🥭",
        "🥙", "🥨", "🥪", "🥦", "🧅", "🧇", "🥬", "🍯", "🍡", "🥖",
    ];

    // Return if the requested amount is greater than the amount of emojis in the list
    if emoji_list.len() < amount {
        return None;
    }

    // Select the range from 0 to emoji_list.len() - 1
    let range = Uniform::from(0..emoji_list.len());
    let mut rng = rand::thread_rng();
    
    let mut result_set : HashSet<String> = HashSet::new();

    // Check after every insert if the requested amount was collected
    // Since the results are stored in a set, they are guaranteed to be unique
    while result_set.len() < amount {
        result_set.insert(
            emoji_list[
                range.sample(&mut rng)
            ]
            .to_string()
        );
    }

    Some(result_set)
}

/**
 * Creates a new vote, stores it in the bot_data and sends the message with the vote information
 * For every element of the options vector it checks if the given String is an internal movie id
 * or movie title. If this is the case the corresponding movie is stored in the option. Otherwise
 * the option is stored as a general option.
 */
pub fn create_vote(bot_data: &mut crate::BotData, title: String, options: Vec<String>) {
    let message = bot_data.message.as_ref().expect("Passing message to create_vote function failed.");

    if user_already_owns_a_vote(bot_data, message.author.id) {
        send_user_already_owns_a_vote_error_message(bot_data);
        return;
    } else {
        let mut vote_options: Vec<VoteOptionEnum> = Vec::with_capacity(options.len());
        if let Some(emojis) = get_random_unique_emojis(vote_options.capacity()) {
            let emojis : Vec<String> = emojis.iter().cloned().collect();
            let mut emoji_idx = 0;
            // Check all options for ids or movie titles
            // Construct the options simultaneously and add them to the temporary vector
            for option in options {
                use crate::movie_behaviour::{Movie, get_movie_id_in_watch_list};

                // If the option starts with prefix id:
                if let Some(movie_id_string) = option.strip_prefix("id:") {
                    // Try to parse the id_string into u32
                    if let Ok(movie_id) = movie_id_string.parse::<u32>() {
                        // Try to get the movie from the watch_list
                        if let Some(watch_list_entry) = bot_data.watch_list.get(&movie_id) {
                            // Finally push the vote option with the movie as cargo
                            vote_options.push(
                                VoteOptionEnum::MovieVoteOption(
                                    VoteOption::<Movie> {
                                        emoji: emojis[emoji_idx].clone(),
                                        cargo: watch_list_entry.movie.clone(),
                                        votes: Vec::new(),
                                    }
                                )
                            );
                        } else {
                            send_movie_not_found_in_watchlist_error_message(bot_data, movie_id_string.to_string());
                            return;
                        }
                    } else {
                        send_wrong_vote_parameter_error_message(bot_data, option);
                        return;
                    }
                } 
                // If the option starts with prefix t:
                else if let Some(movie_title) = option.strip_prefix("t:") {
                    // Try to get the movie id based on the movie title from the watch_list
                    if let Some(movie_id) = get_movie_id_in_watch_list(movie_title, &bot_data.watch_list) {
                        // Try to get the movie from the watch_list
                        if let Some(watch_list_entry) = bot_data.watch_list.get(&movie_id) {
                            // Finally push the vote option with the movie as cargo
                            vote_options.push(
                                VoteOptionEnum::MovieVoteOption(
                                    VoteOption::<Movie> {
                                        emoji: emojis[emoji_idx].clone(),
                                        cargo: watch_list_entry.movie.clone(),
                                        votes: Vec::new(),
                                    }
                                )
                            );
                        } else {
                            send_movie_not_found_in_watchlist_error_message(bot_data, movie_title.to_string());
                            return;
                        }
                    } else {
                        send_movie_not_found_in_watchlist_error_message(bot_data, movie_title.to_string());
                        return;
                    }
                }
                // If the option is a normal vote option
                else {
                    vote_options.push(
                        VoteOptionEnum::GeneralVoteOption(
                            VoteOption::<String> {
                                emoji: emojis[emoji_idx].clone(),
                                cargo: option,
                                votes: Vec::new(),
                            }
                        )
                    );
                }

                emoji_idx += 1;
            }
        } else {
            send_not_enough_emojis_error_message(bot_data);
            return;
        }

        // Create the new instance of the vote struct
        let mut new_vote = Vote {
            creator: message.author.clone(),
            creation_date: message.timestamp,
            title: title,
            options: vote_options,
            message_id: discord::model::MessageId(0),
        };

        // Send the vote details message and assign it to the bot_data
        // If the sending was successful, add the vote to the waiting_for_reaction list
        if let Some(message_id) = send_vote_details_message(bot_data, &mut new_vote) {
            bot_data.wait_for_reaction.push(crate::general_behaviour::WaitingForReaction::Vote(message_id))
        } else {
            send_vote_message_failed_to_send_error_message(bot_data);
        }
    }
}

/**
 * Returns true if the user owns a vote. False otherwise
 */
fn user_already_owns_a_vote(bot_data: &crate::BotData, user_id: discord::model::UserId) -> bool {
    for (_, vote) in bot_data.votes.iter() {
        if vote.creator.id == user_id {
            // User already has an open vote, so return true
            return true;
        }
    }

    false
}

/**
 * Tells the user that he already own a vote
 */
fn send_user_already_owns_a_vote_error_message(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_user_already_owns_a_vote_error_message failed.").channel_id, 
        "",
        |embed| embed
        .title("Du besitzt bereits eine Abstimmung.")
        .description("Bitte beende zunächst die Abstimmung bevor du eine neue eröffnest.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Informs the user, that he tried to create a vote with too many options, which is the reason why it couldn't be created
 */
fn send_not_enough_emojis_error_message(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_not_enough_emojis_error_message failed.").channel_id, 
        "",
        |embed| embed
        .title("Zu viele Vote-Optionen")
        .description("Die Abstimmung konnte nicht erstellt werden, da leider zu viele Optionen hinzugefügt wurden. Versuche es bitte mit weniger Optionen erneut.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Informs the user, that the given movie (either by id or by title) could not be found in the watch_list
 */
fn send_movie_not_found_in_watchlist_error_message(bot_data: &crate::BotData, option_string: String) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_movie_not_found_in_watchlist_error_message failed.").channel_id, 
        "",
        |embed| embed
        .title("Film konnte nicht gefunden werden")
        .description(
            format!("Die Abstimmung konnte nicht erstellt werden, da die Option '{}' nicht in der Filmliste gefunden werden konnte.", option_string).as_str()
        )
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Informs the user, that the given id parameter had the wrong format
 */
fn send_wrong_vote_parameter_error_message(bot_data: &crate::BotData, parameter: String) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_wrong_vote_parameter_error_message failed.").channel_id, 
        "",
        |embed| embed
        .title("Abstimmungsparameter hat falsches Format")
        .description(
            format!("Die Abstimmung konnte nicht erstellt werden, da die Option '{}' das falsche Format für eine ID hat. Bitte verwende nur Zahlen.", parameter).as_str()
        )
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Sends a message including the details to a given vote
 * If the vote is already included in the bot_data (vote.message_id != 0) assigns the resulting
 * message_id to the vote and inserts it into the votes
 * If the vote already exists in the bot_data, updates the key of the corresponding entry
 * 
 * Returns true if the message was sent successfully, false otherwise
 */
pub fn send_vote_details_message(bot_data: &mut crate::BotData, vote: &mut Vote) -> Option<discord::model::MessageId> {
    let embed_description: String = build_vote_embed_description(vote);

    let vote_message = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_vote_details_message failed.").channel_id,
        "",
        |embed| embed
        .title(format!("{}", vote.title).as_str())
        .description(embed_description.as_str())
        .author(|author_builder| 
            if let Some(avatar_url) = vote.creator.avatar_url() {
                author_builder
                .name(vote.creator.name.as_str())
                .icon_url(avatar_url.as_str())
            } else {
                author_builder
                .name(vote.creator.name.as_str())
            }
        )
        .footer(|footer| footer
            .text(
                format!("Um abzustimmen reagiere bitte auf diese Nachricht • {}", 
                    crate::general_behaviour::timestamp_to_string(&vote.creation_date, false)
                ).as_str()
            )
        )
        .color(crate::COLOR_BOT)
    );

    if let Ok(vote_message) = vote_message {
        // Add the reactions to the message
        for vote_option in vote.options.iter() {
            match vote_option {
                VoteOptionEnum::GeneralVoteOption(string_option) => {
                    let _ = bot_data.bot.add_reaction(
                        vote_message.channel_id,
                        vote_message.id,
                        discord::model::ReactionEmoji::Unicode(string_option.emoji.clone())
                    );
                },
                VoteOptionEnum::MovieVoteOption(movie_option) => {
                    let _ = bot_data.bot.add_reaction(
                        vote_message.channel_id,
                        vote_message.id,
                        discord::model::ReactionEmoji::Unicode(movie_option.emoji.clone())
                    );
                }
            }
        }

        // Vote already exists in the bot_data, so remove the previous entry from the bot_data
        if vote.message_id != discord::model::MessageId(0) {
            bot_data.votes.remove(&vote.message_id.0);
        } 

        // Independent of the previous state, set the message_id and insert the (new) vote into bot_data
        vote.message_id = vote_message.id;
        bot_data.votes.insert(vote_message.id.0, vote.clone());

        return Some(vote_message.id);
    } else {
        send_vote_message_failed_to_send_error_message(bot_data);
        return None;
    }
}

/**
 * Constructs the vote description for the embedded message, consisting of general options and/or movie options
 */
fn build_vote_embed_description(vote: &Vote) -> String {
    let mut description = String::new();

    for option in vote.options.iter() {
        match option {
            VoteOptionEnum::GeneralVoteOption(string_option) => {
                description.push_str(
                    format!("\n\n`{}` {} - {}", string_option.votes.len(), string_option.emoji, string_option.cargo).as_str()
                );
            },
            VoteOptionEnum::MovieVoteOption(movie_option) => {
                description.push_str(
                    format!("\n\n`{}` {} - [{}]({})", 
                        movie_option.votes.len(), 
                        movie_option.emoji, 
                        movie_option.cargo.movie_title, 
                        crate::movie_behaviour::get_movie_link(movie_option.cargo.tmdb_id, false)
                    ).as_str()
                )
            }
        }
    }

    description
}

/**
 * Sends an error message that tells the user, that the vote message could not be sent to the channel
 */
fn send_vote_message_failed_to_send_error_message(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_vote_message_failed_to_send_error_message failed.").channel_id, 
        "",
        |embed| embed
        .title("Senden fehlgeschlagen")
        .description("Aus unerklärlichen Gründen ist das Senden der Abstimmungsnachricht leider fehlgeschlagen.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Searches the vote of a user in the votes vector. If a vote was found sends the vote message again.
 * If the user has no vote, sends a message.
 */
pub fn determine_vote_and_send_details_message(bot_data: &mut crate::BotData) {
    let message = bot_data.message.clone().expect("Passing message to determine_vote_and_send_details_message failed.");

    for (_, vote) in bot_data.votes.clone().iter_mut() {
        // If a vote was created by the same user, who sent the message, we found the vote
        if vote.creator.id == message.author.id {
            let previous_message_id = vote.message_id;

            // First remove all reactions on previous vote
            for option in vote.options.iter() {
                let emoji_string = match option {
                    VoteOptionEnum::GeneralVoteOption(general_option) => general_option.emoji.clone(),
                    VoteOptionEnum::MovieVoteOption(movie_option) => movie_option.emoji.clone(),
                };

                let _ = bot_data.bot.delete_reaction(
                    message.channel_id,
                    previous_message_id,
                    None,
                    discord::model::ReactionEmoji::Unicode(emoji_string)
                );
            }

            // Send the vote details message and assign it to the bot_data
            // If the sending was successful, add the vote to the waiting_for_reaction list
            if let Some(message_id) = send_vote_details_message(bot_data, vote) {
                bot_data.wait_for_reaction.push(crate::general_behaviour::WaitingForReaction::Vote(message_id));
                
                // Remove previous wait_for_reaction of previous vote
                for i in 0..bot_data.wait_for_reaction.len() {
                    if let crate::general_behaviour::WaitingForReaction::Vote(some_message_id) = bot_data.wait_for_reaction[i] {
                        if previous_message_id == some_message_id {
                            bot_data.wait_for_reaction.remove(i);
                            break;
                        }
                    }
                }
            } else {
                send_vote_message_failed_to_send_error_message(bot_data);
            }
            return;
        }
    }

    // If the user has not vote, send a message
    send_user_has_no_vote_error_message(bot_data);
}

/**
 * Sends an error message that the user has no vote yet
 */
fn send_user_has_no_vote_error_message(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_user_has_no_vote_error_message failed.").channel_id, 
        "",
        |embed| embed
        .title("Keine Abstimmung")
        .description("Es sieht so aus als ob du aktuell keine Abstimmung besitzt.")
        .color(crate::COLOR_ERROR)
    );
}

/**
 * Updates a vote based on the given reaction
 */
pub fn update_vote(bot_data: &mut crate::BotData, reaction: &discord::model::Reaction, message_id: &u64) {
    // Find the vote in the votes from bot_data
    if let Some(vote) = bot_data.votes.get_mut(message_id) {
        if is_emoji_part_of_vote(vote, reaction) {
            update_user_choice(&bot_data.bot, vote, reaction);
            update_vote_embed(&bot_data.bot, &reaction.channel_id, vote, &reaction.message_id);

            let _ = bot_data.bot.delete_reaction(reaction.channel_id, reaction.message_id, Some(reaction.user_id), reaction.emoji.clone());
        } else {
            send_emoji_not_part_of_vote_info_message(bot_data);
        }
    } else {
        send_vote_not_found_error_message(bot_data, &reaction.user_id);
    }
}

/**
 * Checks whether the given reaction emoji is part of the given vote
 */
fn is_emoji_part_of_vote(vote: &Vote, reaction: &discord::model::Reaction) -> bool {
    // Iterate through all options
    for vote_option_enum in vote.options.iter() {
        if let discord::model::ReactionEmoji::Unicode(emoji_string) = &reaction.emoji {
            let option_emoji = match vote_option_enum {
                VoteOptionEnum::GeneralVoteOption(vote_option) => &vote_option.emoji,
                VoteOptionEnum::MovieVoteOption(vote_option) => &vote_option.emoji,
            };

            // If the emoji strings (unicode) are the same, the emoji is part of that vote
            if *option_emoji == *emoji_string {
                return true;
            }
        }
    }
    
    // If no option with this emoji is found, the emoji is not part of that vote
    false
}

/**
 * Inserts the users choice to the vote if the user has not chosen an option yet
 * Otherwise updates the users choice in the vote by removing the current choice
 * and replacing it with the new one
 */
fn update_user_choice(bot: &discord::Discord, vote: &mut Vote, reaction: &discord::model::Reaction) {
    // Find the user in the options of the vote
    for vote_option_enum in vote.options.iter_mut() {
        let (option_emoji, option_user_list) = match vote_option_enum {
            VoteOptionEnum::GeneralVoteOption(general_option) => (&general_option.emoji, &mut general_option.votes),
            VoteOptionEnum::MovieVoteOption(movie_option) => (&movie_option.emoji, &mut movie_option.votes),
        };

        // If the user_id is found in the votes of this option, store the index, 
        // in case the user needs to be removed from the vote list
        if let Some(idx) = user_id_in_votes(&reaction.user_id, option_user_list) {
            // If the current option has a different emoji than the reaction, that means
            // that the user voted for another option, so remove his vote from this option
            if let discord::model::ReactionEmoji::Unicode(reaction_emoji) = &reaction.emoji {
                if reaction_emoji != option_emoji {
                    // Remove the vote from the user vote list
                    option_user_list.remove(idx);
                } else {
                    // The user has already voted for this option, so send him a private message
                    if let Ok(private_channel) = bot.create_private_channel(reaction.user_id) {
                        let _ = bot.send_embed(
                            private_channel.id, 
                            "",
                            |embed| embed
                                .title("Bereits abgestimmt.")
                                .description("Du hast bereits für diese Option abgestimmt.")
                                .color(crate::COLOR_INFORMATION)
                        );
                    }
                }
            }
        } 
        // The user has not voted yet for this option
        else {
            // If the current option has the same emoji as the reaction, that means the user
            // voted for this option, so add the user id to the votes vector
            if let discord::model::ReactionEmoji::Unicode(reaction_emoji) = &reaction.emoji {
                if reaction_emoji == option_emoji {
                    option_user_list.push(reaction.user_id);
                }
            }
        }
    }
}

/**
 * Returns the index of the user if the user is in the list of votes, None if not.
 */
fn user_id_in_votes(user_id: &discord::model::UserId, vote_user_list: &Vec<discord::model::UserId>) -> Option<usize> {
    for i in 0..vote_user_list.len() {
        if *user_id == vote_user_list[i] {
            return Some(i);
        }
    }

    None
}

/**
 * Edits the embed message of a previously send vote message
 */
fn update_vote_embed(bot: &discord::Discord, channel_id: &discord::model::ChannelId, vote: &Vote, message_id: &discord::model::MessageId) {
    let embed_description: String = build_vote_embed_description(vote);

    let _ = bot.edit_embed(
        *channel_id,
        *message_id,
        |embed| embed
        .title(format!("{}", vote.title).as_str())
        .description(embed_description.as_str())
        .author(|author_builder| 
            if let Some(avatar_url) = vote.creator.avatar_url() {
                author_builder
                .name(vote.creator.name.as_str())
                .icon_url(avatar_url.as_str())
            } else {
                author_builder
                .name(vote.creator.name.as_str())
            }
        )
        .footer(|footer| footer
            .text(
                format!("Um abzustimmen reagiere bitte auf diese Nachricht • {}", 
                    crate::general_behaviour::timestamp_to_string(&vote.creation_date, false)
                ).as_str()
            )
        )
        .color(crate::COLOR_BOT)
    );
}

/**
 * Sends the error message that the vote could not be found in the list of votes
 */
fn send_vote_not_found_error_message(bot_data: &mut crate::BotData, user_id: &discord::model::UserId) {
    if let Ok(private_channel) = bot_data.bot.create_private_channel(*user_id) {
        let _ = bot_data.bot.send_embed(
            private_channel.id, 
            "",
            |embed| embed
            .title("Abstimmung existiert nicht")
            .description("Vielleicht hast du versucht auf eine alte Abstimmung zu reagieren, oder der Nutzer hat die Abstimmung erneut in den Kanal gesendet?")
            .color(crate::COLOR_ERROR)
        );
    }
}

/**
 * Sends the error message, that the given reaction emoji to a given vote is not
 * part of that vote, meaning the user has reacted with an additional emoji.
 * The user should be informed, that the emoji does not change anything in the vote
 * and that the user should react with an emoji that is part of the vote
 */
fn send_emoji_not_part_of_vote_info_message(bot_data: &mut crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.as_ref().expect("Passing message to send_emoji_not_part_of_vote_info_message failed.").channel_id, 
        "",
        |embed| embed
        .title("Emoji ist nicht Teil der Abstimmung")
        .description("Danke für deine Reaktion auf meine Nachricht, aber ich bin verpflichtet dir mitzuteilen, dass dieses Emoji nicht Teil der Abstimmung ist. Falls du eine Stimme abgeben möchtest reagiere bitte mit einem passenden Emoji.")
        .color(crate::COLOR_INFORMATION)
    );
}