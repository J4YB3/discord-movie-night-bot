# discord-movie-night-bot
A bot for managing movies and deciding on which movie should be watched next.

## Using the bot on your own server
Since I can't provide a web server for you to host this bot on, you need to follow these steps:
1. Note that in the process you have to provide personal information to TMDb in order to get your API key. In case you don't want that, you can stop here.
1. Download the source files via git clone or as a zip archive (and unpack them)
1. Create an application on the official discord developers page (discord.com/developers)
1. Attach a bot to the created application under `Settings > Bot` 
1. Copy the Bot-Token to your clipboard
1. Add the token to the source files
    1. Create the file `lib.rs` in the `external_data/src` directory
    1. To this file add the line `pub static DISCORD_TOKEN: &'static str = "<YOUR_TOKEN>";` where you replace `<YOUR_TOKEN>` with the previously copied token. Note that the double quotes are necessary.
1. Get an TMDb API key
    1. Create an TMDb account, if you don't already have one.
    1. Under profile->settings->API create an API key, accepting the terms of use and filling in your personal data into the form.
    1. Once you have created the key, add this line to the file as well, replacing <API_KEY> with your API key: `pub static TMDB_API_KEY: &'static str = <API_KEY>`

In order to compile an executable file you need to have the programming language Rust and its dependencies installed on your system. Because you need to generate your own token and API key I can not provide an executable.

### Important note: 
Publishing the token will open the bot to hacking attacks, since everybody with the token can potentially run dangerous programs on the bot. Do never publish the token anywhere.

## Inviting the bot to your server
After the executable was created you just need to invite the bot to your server.
Following this link (discordapi.com/permissions.html#257088) will lead you to a permissions calculator. On this page you just need to paste the Client ID of your Discord application into the field at the bottom. The Client ID can be found on the Discord developers page (where you created the Discord bot application) under the section `OAuth2`.  
After you inserted the Client ID you can follow the link at the bottom of the page, which will redirect you to a Discord page asking you to log into your Discord account. If you are logged in, the page will prompt you to enter a server to add the bot to.

## Starting the executable
Once all these steps are completed you can start the executable. The bot will wake up and should now be online on your server.
