use insta::assert_snapshot;
use metadata_search_engine_rs::models::AggregatedResult;
use ratatui::{Terminal, backend::TestBackend};
use search_tui::app::{App, Mode};
use search_tui::ui::ui;

fn make_result(title: &str, url: &str) -> AggregatedResult {
    AggregatedResult {
        title: title.to_string(),
        url: url.to_string(),
        snippet: None,
        engines: vec!["ddg".to_string()],
        score: 1.0,
    }
}

fn make_result_with_snippet(title: &str, url: &str, snippet: &str) -> AggregatedResult {
    AggregatedResult {
        snippet: Some(snippet.to_string()),
        ..make_result(title, url)
    }
}

fn render(app: &mut App) -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, app)).unwrap();
    terminal
}

// ---------------------------------------------------------------------------
// Snapshot tests — run `cargo insta review` after intentional UI changes
// ---------------------------------------------------------------------------

#[test]
fn snapshot_input_mode_empty() {
    let mut app = App::new();
    let terminal = render(&mut app);
    assert_snapshot!(terminal.backend());
}

#[test]
fn snapshot_input_mode_with_text() {
    let mut app = App::new();
    app.input = "ratatui widgets".to_string();
    let terminal = render(&mut app);
    assert_snapshot!(terminal.backend());
}

#[test]
fn snapshot_loading_mode() {
    let mut app = App::new();
    app.mode = Mode::Loading;
    let terminal = render(&mut app);
    assert_snapshot!(terminal.backend());
}

#[test]
fn snapshot_error_mode() {
    let mut app = App::new();
    app.mode = Mode::Error("All engines failed to respond.".to_string());
    let terminal = render(&mut app);
    assert_snapshot!(terminal.backend());
}

#[test]
fn snapshot_browse_single_result() {
    let mut app = App::new();
    app.set_results(vec![make_result(
        "Tokio — async runtime for Rust",
        "https://tokio.rs",
    )]);
    let terminal = render(&mut app);
    assert_snapshot!(terminal.backend());
}

#[test]
fn snapshot_browse_multiple_results() {
    let mut app = App::new();
    app.set_results(vec![
        make_result("Tokio async runtime", "https://tokio.rs"),
        make_result("Ratatui TUI framework", "https://ratatui.rs"),
        make_result("Serde serialization", "https://serde.rs"),
    ]);
    let terminal = render(&mut app);
    assert_snapshot!(terminal.backend());
}

#[test]
fn snapshot_browse_result_with_snippet() {
    let mut app = App::new();
    app.set_results(vec![make_result_with_snippet(
        "Serde",
        "https://serde.rs",
        "A framework for serializing and deserializing Rust data structures efficiently.",
    )]);
    let terminal = render(&mut app);
    assert_snapshot!(terminal.backend());
}

#[test]
fn snapshot_browse_second_item_selected() {
    let mut app = App::new();
    app.set_results(vec![
        make_result("Tokio async runtime", "https://tokio.rs"),
        make_result("Ratatui TUI framework", "https://ratatui.rs"),
        make_result("Serde serialization", "https://serde.rs"),
    ]);
    app.next();
    let terminal = render(&mut app);
    assert_snapshot!(terminal.backend());
}
