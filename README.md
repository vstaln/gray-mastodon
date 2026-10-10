<div align="center">
  <img alt="gray-mastodon" src="assets/icon.svg" width="120" height="120" />
  <h1>gray-mastodon</h1>
  <p><strong>Mastodon tools — post toots, read notifications, timelines, and search.</strong></p>
  <p>
    <a href="https://gray.alignment.id">Website</a> ·
    <a href="https://github.com/vstaln/gray-mastodon">Source</a> ·
    <a href="https://github.com/vstaln/gray">gray</a>
  </p>
  <p>
    <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-1c1c20?style=flat-square&labelColor=0a0a0b" /></a>
    <a href="https://www.rust-lang.org"><img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-rust-1c1c20?style=flat-square&labelColor=0a0a0b&logo=rust&logoColor=d4a373" /></a>
    <img alt="gray plugin" src="https://img.shields.io/badge/gray-plugin-1c1c20?style=flat-square&labelColor=0a0a0b&color=7aa2f7" />
  </p>
</div>

<br/>
```bash
gray plugin install mastodon
```

`gray-mastodon` connects the agent to any Mastodon (or compatible
ActivityPub) instance over the standard REST API.

## Setup

```sh
mastodon setup --instance https://mastodon.social --token <app-token>
mastodon status
```

Config: `~/.gray/mastodon/config.json` (`{"instance","token"}`) or env
`MASTODON_INSTANCE` / `MASTODON_TOKEN`. The token is an application access
token (Preferences → Development → New Application, `read write` scopes).

## Tools

| Tool | What it does |
|---|---|
| `mastodon_post` | Post a status — `text`, `visibility` (public/unlisted/private/direct), `reply_to`, `spoiler` (CW). |
| `mastodon_notifications` | Recent notifications — `limit`. |
| `mastodon_timeline` | Read a timeline — `scope` (home/local/federated/tag), `limit`. |
| `mastodon_search` | Search statuses/accounts — `query`, `limit`. |

## Build from source

```sh
cargo build --release
gray plugin check ./target/release/mastodon
```

## License

MIT
