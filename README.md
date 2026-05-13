# search-tui

A terminal UI for the metadata search engine, built with [ratatui](https://ratatui.rs) and [metadata-search-engine-rs](https://crates.io/crates/metadata-search-engine-rs). Fans out queries to DuckDuckGo, Brave, and Startpage concurrently and displays RRF-ranked results you can browse and open directly from the terminal.

## Running

From the workspace root:

```bash
cargo install search-tui
sui "your query"
```

## Terminal UI

![sui TUI](assets/tui_screenshots.png)
