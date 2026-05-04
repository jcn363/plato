# Article Fetcher

Plato supports multiple read-later services for saving and reading web articles offline.

## Supported Services

- **Pocket** (getpocket.com) - Full API integration with OAuth
- **Instapaper** (instapaper.com) - Full API with folders and highlights
- **Wallabag** - Self-hosted or cloud article saving

The legacy *wallabag* article fetcher binary is distributed in the release archive at `bin/article_fetcher`.

## New Integration (May 2026)

Native Pocket and Instapaper integration is now built into `plato-core`:

- `crates/core/src/pocket.rs` - Full Pocket API client
- `crates/core/src/instapaper.rs` - Full Instapaper API client
- `crates/core/src/article.rs` - Unified article data structures

### Features

- **Sync**: Automatic background sync of saved articles
- **Offline reading**: Download article content for offline access
- **Tag management**: Organize articles with tags
- **Archive**: Mark articles as read and archive them
- **Highlights**: Sync and export highlights (Instapaper, Readwise, Obsidian)
- **Progress sync**: Resume reading position across devices
- **Folder support** (Instapaper): Organize into folders

### Configuration

Configure in your `Settings.toml`:

```toml
[pocket]
consumer_key = "your-consumer-key"
access_token = "your-access-token"
auto_sync = true
sync_progress = true
archive_after_reading = false

[instapaper]
username = "your-email@example.com"
password = "your-password"
auto_sync = true
sync_progress = true
archive_after_reading = false
```

## Legacy Wallabag Fetcher

### Wallabag Configuration

Rename `Settings-sample.toml` to `Settings.toml` and fill it out.

The fetcher manages a `.session.json` file that you shouldn't modify or remove.

## Usage

In the library menu:

- Select *Library → On Board*.
- Select *Toggle Select → Articles* (the downloaded articles are saved in the hook's *path*).

If the *Toggle Select* sub-menu is missing, [add the relevant hook](HOOKS.md).

## Build

The default article fetcher can be built with:

```sh
cargo +nightly build --profile release-minsized -Z build-std=std,panic_abort \
                     --target arm-unknown-linux-gnueabihf \
                     --bin article_fetcher -p fetcher
```
