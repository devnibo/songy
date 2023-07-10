# songy

This is a telegram bot written in [rust](https://www.rust-lang.org/) using the [frankenstein](https://github.com/ayrat555/frankenstein) telegram bot library.

## what does it do?

This bot provides an interface to files. It is meant to be used as a digital songbook.

<img src="https://github.com/devnibo/songy/raw/master/example.gif" alt="example" height="600" />

### bot commands

To let telegram know what commands the bot provides you have to set these in the [BotFather](https://telegram.me/BotFather).
The `/list` command lists all available files in the provided --songs-path recursively. There is one thing you can do for additional structuring. Suppose you have provided the path `/var/songs` as the --songs-path. If you create the subfolder `/var/songs/en` then the command `/en` will be available in the bot to list only files/songs recursively in that subfolder. That way you could organize your songs in different languages.

## installation

1. Download the [latest release](https://github.com/devnibo/songy/releases) executable
2. Decide about the configuration
	1. [Create a telegram bot](https://telegram.me/BotFather) to obtain the bot api token (--token)
	2. Which language do you want the bot to speak? english, german or moldovan (--lang)
	3. Where do you store the files that the bot uses? (--songs-path)
3. Start the bot: `./songs --token <api_token> --lang <en|de|md> --songs-path <full/path/to/songs/folder>`

### setup systemd service under linux

As the bot has to run endlessly you probably want to create some sort of background service. Here's a simple solution I use.

1. `sudo touch /etc/systemd/system/songy.service`
2. Copy this into `songy.service`
```
[Unit]
Description=Digital song book

[Service]
ExecStart=/full/path/to/songy/executable --token <api_token> --songs-path <full/path/to/songs/folder> --lang <en|de|md>
Restart=on-failure

[Install]
WantedBy=default.target
```
3. `systemctl start songy`
