# yt-notify
first rust project don't judge it

it does not watch. you will have to write that yourself

so `cargo build`

```bash
Usage: yt-notify [-w|--webhook-url=DISCORD_URL] [-u|--user-id=DISCORD_ID] [-b|--open-browser] <YOUTUBE_CHANNEL_ID>
```

example:
```bash
yt-notify -w https://discord.com/api/webhooks/0000000000/aaaaaa-000000 -u 0000000000 -b UCuLpYAaxNfBdODBRd8pOfXQ
# posts to a discord webhook, pinging user <@0000000000>, opens in browser, IF UCuLpYAaxNfBdODBRd8pOfXQ posts.
```
