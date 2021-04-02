# discord-movie-night-bot
A bot for managing movies and deciding on which movie should be watched next.

## Using the bot on your own server
Since I can't provide a web server for you to host this bot on, you need to follow these steps:
1. Download the source files via git clone or as a zip archive (and unpack them)
1. Create an application on the official discord developers page (discord.com/developers)
1. Attach a bot to the created application under `Settings > Bot` 
1. Copy the Bot-Token to your clipboard
1. Add the token to the source files
    1. Create the file `lib.rs` in the `discord_data/src` directory
    1. To this file add the line `pub static TOKEN: &'static str = "<YOUR_TOKEN>";` where you replace `<YOUR_TOKEN>` with the previously copied token. Note that the double quotes are necessary.

In order to compile an executable file you need to have the programming language Rust and its dependencies installed on your system. Because you need to generate your own token I can not provide an executable. Publishing the token will open the bot to hacking attacks, since everybody with the token can run dangerous programs on the bot.