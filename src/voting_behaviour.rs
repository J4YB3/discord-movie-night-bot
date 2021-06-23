use rand::distributions::{Distribution, Uniform};
use std::collections::HashSet;
use crate::send_message;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum VoteOptionEnum {
    GeneralVoteOption(VoteOption<String>),
    MovieVoteOption(VoteOption<crate::movie_behaviour::Movie>),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VoteOption<T> {
    emoji: String,
    cargo: T,
    votes: Vec<discord::model::UserId>,
}

#[derive(Clone, Serialize, Deserialize)]
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
pub fn create_vote(bot_data: &mut crate::BotData, title: String, options: Vec<String>, is_movie_vote: bool) {
    let message = bot_data.message.as_ref().expect("Passing message to create_vote function failed.");

    let creator = if is_movie_vote { bot_data.bot_user.clone() } else { message.author.clone() };

    if user_already_owns_a_vote(bot_data, creator.id) {
        // The case, that a random_movie_vote already exists, is covered by the create_random_movie_vote function

        send_message::user_already_owns_a_vote_error(bot_data);
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
                            send_message::movie_not_found_in_watchlist_error(bot_data, movie_id_string.to_string());
                            return;
                        }
                    } else {
                        send_message::wrong_vote_parameter_error(bot_data, option);
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
                            send_message::movie_not_found_in_watchlist_error(bot_data, movie_title.to_string());
                            return;
                        }
                    } else {
                        send_message::movie_not_found_in_watchlist_error(bot_data, movie_title.to_string());
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
            send_message::not_enough_emojis_error(bot_data);
            return;
        }

        // Create the new instance of the vote struct
        let mut new_vote = Vote {
            creator: creator.clone(),
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
            send_message::vote_message_failed_to_send_error(bot_data);
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
 * Sends a message including the details to a given vote
 * If the vote is already included in the bot_data (vote.message_id != 0) assigns the resulting
 * message_id to the vote and inserts it into the votes
 * If the vote already exists in the bot_data, updates the key of the corresponding entry
 * 
 * Returns Some(message_id) if the message was sent successfully, None otherwise
 */
pub fn send_vote_details_message(bot_data: &mut crate::BotData, vote: &mut Vote) -> Option<discord::model::Message> {
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
        // Remove the reactions from the previous message
        remove_all_reactions_on_previous_vote(bot_data, vote, (&vote_message.channel_id, &vote.message_id));

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

        return Some(vote_message);
    } else {
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
 * Searches the vote of a user in the votes vector. If a vote was found sends the vote message again.
 * If the user has no vote, sends a message.
 */
pub fn determine_vote_and_send_details_message(bot_data: &mut crate::BotData, other_user_id: Option<u64>) {
    let message = bot_data.message.clone().expect("Passing message to determine_vote_and_send_details_message failed.");

    // If another user id was given to the function, set the comparing_user_id to that one, otherwise set as message.author.id
    let comparing_user_id = if let Some(user_id) = other_user_id { discord::model::UserId(user_id) } else { message.author.id };

    for (_, vote) in bot_data.votes.clone().iter_mut() {
        // If a vote was created by the same user, who sent the message, we found the vote
        if vote.creator.id == comparing_user_id {
            let previous_message_id = vote.message_id;

            // First remove all reactions on previous vote
            remove_all_reactions_on_previous_vote(bot_data, vote, (&message.channel_id, &previous_message_id));

            // Send the vote details message and assign it to the bot_data
            // If the sending was successful, add the vote to the waiting_for_reaction list
            if let Some(message_id) = send_vote_details_message(bot_data, vote) {
                bot_data.wait_for_reaction.push(crate::general_behaviour::WaitingForReaction::Vote(message_id));
                
                remove_previous_vote_from_wait_for_reaction(bot_data, &previous_message_id);
            } else {
                send_message::vote_message_failed_to_send_error(bot_data);
            }
            return;
        }
    }

    // If the user has not vote, send a message
    if other_user_id.is_none() {
        send_message::user_has_no_vote_error(bot_data);
    } else {
        send_message::other_user_has_no_vote_error(bot_data);
    }
}

/**
 * Iterates through all options of the vote and removes all reactions of the previous vote message
 */
pub fn remove_all_reactions_on_previous_vote(bot_data: &crate::BotData, vote: &Vote, channel_and_message_id: (&discord::model::ChannelId, &discord::model::MessageId)) {
    // Iterate through all options of the vote
    for option in vote.options.iter() {
        let emoji_string = match option {
            VoteOptionEnum::GeneralVoteOption(general_option) => general_option.emoji.clone(),
            VoteOptionEnum::MovieVoteOption(movie_option) => movie_option.emoji.clone(),
        };

        let _ = bot_data.bot.delete_reaction(
            *channel_and_message_id.0,
            *channel_and_message_id.1,
            None,
            discord::model::ReactionEmoji::Unicode(emoji_string)
        );
    }
}

/**
 * Finds the previous vote message in the wait_for_reaction vector of bot_data and removes the entry
 */
fn remove_previous_vote_from_wait_for_reaction(bot_data: &mut crate::BotData, previous_message_id: &discord::model::MessageId) {
    // Remove previous wait_for_reaction of previous vote
    for i in 0..bot_data.wait_for_reaction.len() {
        if let crate::general_behaviour::WaitingForReaction::Vote(some_message) = &bot_data.wait_for_reaction[i] {
            if *previous_message_id == some_message.id {
                bot_data.wait_for_reaction.remove(i);
                break;
            }
        }
    }
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

            let _ = bot_data.bot.delete_reaction(
                reaction.channel_id, 
                reaction.message_id, 
                Some(reaction.user_id), 
                reaction.emoji.clone()
            );
        } else {
            send_message::emoji_not_part_of_vote_info(bot_data);
        }
    } else {
        send_message::vote_not_found_error(bot_data, &reaction.user_id);
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
 * Removes the vote from the bot_data and manages all other dependencies
 * Sends a message summarizing the result of the vote
 */
pub fn close_vote(bot_data: &mut crate::BotData) {
    let message = bot_data.message.clone().expect("Passing message to determine_vote_and_send_details_message failed.");

    for (_, vote) in bot_data.votes.clone().iter_mut() {
        // If a vote was created by the same user, who sent the message, we found the vote
        if vote.creator.id == message.author.id {
            let previous_message_id = vote.message_id;

            // First remove all reactions on previous vote
            remove_all_reactions_on_previous_vote(bot_data, vote, (&message.channel_id, &previous_message_id));

            // Send the vote summary message
            if let Some(_) = send_vote_summary_message(bot_data, vote) {
                remove_previous_vote_from_wait_for_reaction(bot_data, &previous_message_id);
                let _ = bot_data.votes.remove(&previous_message_id.0);
            } else {
                send_message::vote_message_failed_to_send_error(bot_data);
            }
            return;
        }
    }

    // If the user has not vote, send a message
    send_message::user_has_no_vote_error(bot_data);
}

/**
 * Sends the vote summary
 */
fn send_vote_summary_message(bot_data: &crate::BotData, vote: &Vote) -> Option<discord::model::MessageId> {
    let mut embed_description = String::from(format!("**{}**", vote.title));
    embed_description.push_str(build_vote_embed_description(vote).as_str());

    if let Ok(message) = bot_data.bot.send_embed(
        bot_data.message.clone().expect("Passing message to send_vote_summary_message failed.").channel_id,
        "",
        |embed| embed
            .title("Abstimmungsergebnisse")
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
            .color(crate::COLOR_SUCCESS)
    ) {
        Some(message.id)
    } else {
        None
    }
}

/** 
 * Updates the movie limit per user and sends an info message
 */
pub fn set_movie_vote_limit(bot_data: &mut crate::BotData, new_limit: u32) {
    use crate::general_behaviour;

    let message = bot_data.message.clone().expect("Passing of message to set_movie_limit failed.");
    
    if !general_behaviour::is_user_administrator(bot_data, message.author.id) {
        return send_message::insufficient_permissions_error(bot_data);
    }

    let old_limit = bot_data.movie_vote_limit;
    bot_data.movie_vote_limit = new_limit;

    let _ = bot_data.bot.send_embed(
        message.channel_id,
        "",
        |embed| embed
            .title("Filmlimit für Abstimmungen aktualisiert")
            .description(format!("Das Filmlimit für Filmabstimmungen wurde von `{}` auf `{}` geändert.", old_limit, new_limit).as_str())
            .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Sends a message showing the current movie vote limit
 */
pub fn show_movie_vote_limit(bot_data: &crate::BotData) {
    let _ = bot_data.bot.send_embed(
        bot_data.message.clone().expect("Passing of message to show_movie_vote_limit function failed.").channel_id,
        "",
        |embed| embed
            .title("Filmlimit für Abstimmungen")
            .description(
                format!("Das aktuelle Filmlimit für Abstimmungen beträgt `{}` {}.", 
                    bot_data.movie_vote_limit, 
                    if bot_data.movie_vote_limit == 1 {"Film"} else {"Filme"}
                )
                .as_str()
            )
            .color(crate::COLOR_INFORMATION)
    );
}

/**
 * Creates a new random movie vote with the given optional limit
 * The movies which are longest on the list have a greater chance of getting selected
 */
pub fn create_random_movie_vote(bot_data: &mut crate::BotData, optional_limit: Option<u32>) {
    if user_already_owns_a_vote(bot_data, bot_data.bot_user.id) {
        if optional_limit.is_some() {
            send_message::there_is_already_a_random_movie_vote_information(bot_data);
        }

        send_random_movie_vote_again(bot_data);
        return;
    }

    use rand::{seq::IteratorRandom, thread_rng, Rng};
    use crate::movie_behaviour;

    // Get the three (or less) movies that have earliest creation date (lowest id)
    let mut earliest_movie_ids_vec : Vec<&u32> = movie_behaviour::get_three_earliest_movie_ids(bot_data);
    earliest_movie_ids_vec.reverse();
    
    let limit = get_movie_limit_or_optional_limit_as_usize(bot_data, optional_limit);
    if limit.is_none() {
        return;
    }

    let limit = limit.unwrap();

    let mut rng = thread_rng();

    let mut random_movies : Vec<&u32> = bot_data.watch_list
        .iter()
        .filter(|(_, entry)| entry.status.is_watch_list_status())
        .choose_multiple(&mut rng, limit)
        .iter()
        .map(|(id, _)| *id)
        .collect();

    let mut removed_earliest_ids : Vec<u32> = Vec::new();

    // Remove all duplicates
    for i in 0..earliest_movie_ids_vec.len() {
        let idx = i - removed_earliest_ids.len();
        if random_movies.contains(&earliest_movie_ids_vec[idx]) {
            removed_earliest_ids.push(*earliest_movie_ids_vec[idx]);
            earliest_movie_ids_vec.remove(idx);
        }
    }

    // Swap the random movies with earliest movies by chance. Skips the earliest movies that already are in the vote
    for id in random_movies.iter_mut() {
        if !removed_earliest_ids.contains(id) && rng.gen_bool(0.4) {
            let earliest_movie = earliest_movie_ids_vec.pop();

            match earliest_movie {
                None => break,
                Some(movie_id) => *id = movie_id
            };
        }
    }

    let options_vec : Vec<String> = random_movies.iter().map(|x| format!("id:{}", x)).collect();

    create_vote(bot_data, String::from("Nächster Film"), options_vec, true);
}

/**
 * Returns the optional limit if it is_some, or the set limit from bot data, each converted to usize
 */
fn get_movie_limit_or_optional_limit_as_usize(bot_data: &crate::BotData, optional_limit: Option<u32>) -> Option<usize> {
    use std::convert::TryInto;
    
    let limit = match optional_limit {
        Some(limit) => limit,
        None => bot_data.movie_vote_limit,
    };
    let limit = limit.try_into();

    if limit.is_err() {
        send_message::unknown_error_occured(bot_data, 100);
        return None;
    } else {
        return Some(limit.unwrap());
    }
}

/**
 * Finds the random movie vote in the votes and sends the vote message again
 * If there is no random movie vote, sends an error message that there is no
 * random_movie_vote
 */
fn send_random_movie_vote_again(bot_data: &mut crate::BotData) {
    // Find the random movie vote in the votes
    if let Some(message_id) = find_random_movie_vote(bot_data) {
        // Extract the vote as mutable from the bot_data
        // This can not panic, since the vote was found by 'find_random_movie_vote'
        let mut the_vote = bot_data.votes.get_mut(&message_id.0).unwrap().clone();

        // Send the vote details message and assign it to the bot_data
        // If the sending was successful, add the vote to the waiting_for_reaction list
        if let Some(message_id) = send_vote_details_message(bot_data, &mut the_vote) {
            bot_data.wait_for_reaction.push(crate::general_behaviour::WaitingForReaction::Vote(message_id))
        } else {
            send_message::vote_message_failed_to_send_error(bot_data);
        }
    } else {
        send_message::no_random_movie_vote_exists_error(bot_data);
    }
}

/**
 * Finds the random movie vote in the bot data and returns its MessageId, or None if the vote couldn't be found
 */
fn find_random_movie_vote(bot_data: &mut crate::BotData) -> Option<discord::model::MessageId> {
    let bot_user_id = bot_data.bot_user.id;

    // Find the random movie vote in the votes
    if let Some(random_movie_vote) = bot_data.votes.iter_mut()
        .find(|(_, vote)| vote.creator.id == bot_user_id) 
    {
        return Some(discord::model::MessageId(*random_movie_vote.0));
    } else {
        return None;
    }
}

pub fn close_random_movie_vote(bot_data: &mut crate::BotData) {
    // Find the random movie vote in the votes
    if let Some(message_id) = find_random_movie_vote(bot_data) {
        // Extract the vote from the bot_data
        // This can not panic, since the vote was found by 'find_random_movie_vote'
        let vote = &bot_data.votes.get(&message_id.0).unwrap().clone();

        let previous_message_id = vote.message_id;
        let message = bot_data.message.clone().expect("Passing of message to close_random_movie_vote failed.");

        // First remove all reactions on previous vote
        remove_all_reactions_on_previous_vote(bot_data, vote, (&message.channel_id, &previous_message_id));

        // Send the vote summary message
        if let Some(_) = send_random_movie_vote_summary_message(bot_data, vote) {
            remove_previous_vote_from_wait_for_reaction(bot_data, &previous_message_id);
            let _ = bot_data.votes.remove(&previous_message_id.0);
        } else {
            send_message::vote_message_failed_to_send_error(bot_data);
        }
        return;
    }

    // If the user has not vote, send a message
    send_message::user_has_no_vote_error(bot_data);
}

fn send_random_movie_vote_summary_message(bot_data: &mut crate::BotData, vote: &Vote) -> Option<discord::model::MessageId> {
    if let Some(movie_vote_winner) = determine_movie_vote_winner(vote) {
        let _ = bot_data.bot.send_embed(
            bot_data.message.clone().expect("Passing message to send_random_movie_vote_summary_message failed.").channel_id,
            "",
            |embed| embed
                .title("Gewinner")
                .description("Der folgende Film hat die Abstimmung gewonnen:")
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
                .color(crate::COLOR_SUCCESS)
        );

        use crate::movie_behaviour::find_id_by_tmdb_id;

        // Try to get the id of the winner inside the watch list
        if let Some(watch_list_id_of_winner) = find_id_by_tmdb_id(movie_vote_winner.cargo.tmdb_id, &bot_data.watch_list) {
            // If the id was found, try to retreive the entry
            if let Some(movie_entry) = bot_data.watch_list.get(&watch_list_id_of_winner) {
                // If this also worked, send the message
                if let Ok(message) = send_message::movie_information(bot_data, movie_entry, false, false, true) {
                    // If the message could be sent, add the reactions to the bot_data
                    let _ = bot_data.bot.add_reaction(
                        message.channel_id,
                        message.id,
                        discord::model::ReactionEmoji::Unicode(String::from("✅"))
                    );
        
                    let _ = bot_data.bot.add_reaction(
                        message.channel_id,
                        message.id,
                        discord::model::ReactionEmoji::Unicode(String::from("❎"))
                    );
            
                    bot_data.wait_for_reaction.push(crate::general_behaviour::WaitingForReaction::AddMovieToWatched(message.clone(), movie_entry.movie.clone()));

                    // Now return the message id
                    return Some(message.id);
                } else {
                    // If the message couldn't be sent, send an error message
                    send_message::sending_of_movie_information_message_failed_error(bot_data);
                }
            } 
            // If the entry couldn't be found, send an error message
            else {
                send_message::movie_not_found_in_watchlist_error(bot_data, movie_vote_winner.cargo.movie_title);
            }
        } 
        // If the id couldn't be found, send an error message
        else {
            send_message::movie_not_found_in_watchlist_error(bot_data, movie_vote_winner.cargo.movie_title);
        }
    } 
    // If there were no movie votes to find the winner, send an error message
    else {
        send_message::no_movie_vote_options_in_movie_vote_error(bot_data);
    }

    return None;
}

fn determine_movie_vote_winner(vote: &Vote) -> Option<VoteOption<crate::movie_behaviour::Movie>> {
    let only_movie_options : Vec<&VoteOption<crate::movie_behaviour::Movie>> = vote.options.iter()
        .filter_map(|option| match option {
            VoteOptionEnum::GeneralVoteOption(_) => None,
            VoteOptionEnum::MovieVoteOption(movie_option) => Some(movie_option),
        })
        .collect();

    if let Some(max_votes) = only_movie_options.iter()
        .map(|option| option.votes.len())
        .max() 
    {
        use rand::seq::IteratorRandom;
        let mut rng = rand::thread_rng();

        if let Some(&random_movie_option_with_max_vote) = only_movie_options.iter()
            .filter(|option| option.votes.len() == max_votes)
            .choose(&mut rng)
        {
            return Some(random_movie_option_with_max_vote.clone());
        }
    }

    return None;
}