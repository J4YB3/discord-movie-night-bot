extern crate discord_data;
use discord::{self, Discord as Discord, model as Model};

fn main() {
    let bot = Discord::from_bot_token(discord_data::TOKEN).expect("Bot creation from token failed");

    let (mut connection, _ready_event) = bot.connect().expect("Establishing connecting to server failed");

    loop {
        match connection.recv_event() {
            Ok(Model::Event::MessageCreate(message)) => {
                println!("{} says: {}", message.author.name, message.content);
                if message.content == "!quit" {
                    let _ = bot.send_message(
                        message.channel_id,
                        "Quitting. Bye bye.",
                        "",
                        false,
                    );
                    break;
                }
            },
            Ok(_) => (),
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}", code, body);
            },
            Err(err) => println!("Received error: {}", err),
        }
    }

    let _ = connection.shutdown();
}
