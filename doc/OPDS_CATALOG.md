# OPDS Catalog Support

Plato now includes support for OPDS (Open Publication Distribution System) catalogs, allowing you to browse and download books directly from online sources like Feedbooks, Project Gutenberg, and Standard Ebooks.

## Features

- **Built-in Catalogs**: Pre-configured access to popular free ebook repositories.
- **Hierarchical Browsing**: Navigate through catalog sections and sub-sections.
- **Asynchronous Downloads**: Books are downloaded in the background to avoid blocking the UI.
- **Automatic Library Integration**: Downloaded books are saved to your `Downloads` folder and automatically imported into your library.
- **Configurable**: Add or remove catalogs via `settings.toml`.

## Usage

1. Open the main menu on the Home view.
2. Navigate to **Applications** -> **OPDS Catalog**.
3. Select a catalog to browse.
4. Select a book to download.
5. Once the download is complete, a notification will appear, and the book will be available in your library's `Downloads` directory.

## Architecture

The OPDS implementation is split into two parts:

### 1. Core Parser (`crates/core/src/opds.rs`)

- Uses `reqwest` for fetching XML/Atom feeds.
- Uses `quick-xml` for efficient stream-based parsing.
- Extracts metadata (title, summary) and links (sub-catalogs, acquisitions).

### 2. UI View (`crates/core/src/view/opds/mod.rs`)

- Implements the `View` trait.
- Manages a URL stack for navigation (Back button support).
- Spawns background threads for book downloads using `std::thread`.
- Uses the project's event system for notifications and library re-indexing.

## Configuration

You can customize the available catalogs in `settings.toml` under the `[opds]` section:

```toml
[opds]
catalogs = [
    { name = "Standard Ebooks", url = "https://standardebooks.org/opds/all" },
    { name = "Feedbooks", url = "https://www.feedbooks.com/publicdomain/browse.atom" },
    { name = "Project Gutenberg", url = "https://m.gutenberg.org/ebooks.opds/" }
]
```

## Future Improvements

- Search support within catalogs.
- Support for authenticated catalogs (Basic Auth).
- Paginated catalog results.
- Cover image previews in the catalog view.
