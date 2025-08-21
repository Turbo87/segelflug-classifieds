segelflug-classifieds
==============================================================================

Segelflug.de Kleinanzeigen Telegram Bot

This project implements a basic [Telegram] bot that regularly polls the
[Segelflug.de Kleinanzeigen] page for new items and sends them to a public
Telegram channel. The bot is available as a [Home Assistant] addon with 
support for [Raspberry Pi] and other ARM devices.

[Telegram]: https://telegram.org
[Segelflug.de Kleinanzeigen]: https://www.segelflug.de/osclass/
[Home Assistant]: https://www.home-assistant.io/
[Raspberry Pi]: https://www.raspberrypi.org


Usage
-------------------------------------------------------------------------------

```
$ segelflug-classifieds --help 

USAGE:
    segelflug-classifieds [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -w, --watch      Run continuously and poll the server in random intervals

OPTIONS:
        --max-time <max-time>
            Maximum time to wait between server requests (in minutes) [default: 30]

        --min-time <min-time>
            Minimum time to wait between server requests (in minutes) [default: 10]

        --telegram-chat-id <telegram-chat-id>
            Telegram chat ID [env: TELEGRAM_CHAT_ID] [default: @segelflug_classifieds]

        --telegram-token <telegram-token>        
            Telegram bot token [env: TELEGRAM_TOKEN]
```

If no Telegram token is provided the application will only output the new items
on the console, if a token is provided they will also be sent to the 
configurable Telegram channel.

In addition to these options a `SENTRY_DSN` environment variable can also be set
to enable error reporting on [Sentry.io].

[Sentry.io]: https://sentry.io/


Deployment
-------------------------------------------------------------------------------

### Home Assistant Addon (Recommended)

The easiest way to run this bot is as a Home Assistant addon:

1. Add this repository to your Home Assistant addon store:
   ```
   https://github.com/Turbo87/segelflug-classifieds
   ```

2. Install the "Segelflug.de Kleinanzeigen Bot" addon

3. Configure the addon with your Telegram bot token:
   - Get a token from [@BotFather](https://t.me/BotFather) on Telegram
   - Add the token to the addon configuration
   - Optionally configure a Sentry DSN for error reporting

4. Start the addon

The addon will automatically download the latest binary on startup and handle updates.


License
-------------------------------------------------------------------------------

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


Contribution
-------------------------------------------------------------------------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.