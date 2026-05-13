use metadata_search_engine_rs::models::AggregatedResult;
use search_tui::app::{App, Mode};

fn make_result(title: &str, url: &str) -> AggregatedResult {
    AggregatedResult {
        title: title.to_string(),
        url: url.to_string(),
        snippet: None,
        engines: vec!["test".to_string()],
        score: 1.0,
    }
}

fn make_result_with_snippet(title: &str, url: &str, snippet: &str) -> AggregatedResult {
    AggregatedResult {
        snippet: Some(snippet.to_string()),
        ..make_result(title, url)
    }
}

// ---------------------------------------------------------------------------
// App::new
// ---------------------------------------------------------------------------

#[test]
fn new_starts_in_input_mode() {
    let app = App::new();
    assert!(matches!(app.mode, Mode::Input));
}

#[test]
fn new_has_empty_input() {
    let app = App::new();
    assert!(app.input.is_empty());
}

#[test]
fn new_has_no_results() {
    let app = App::new();
    assert!(app.results.is_empty());
}

#[test]
fn new_has_no_list_selection() {
    let app = App::new();
    assert!(app.list_state.selected().is_none());
}

// ---------------------------------------------------------------------------
// App::set_results
// ---------------------------------------------------------------------------

#[test]
fn set_results_transitions_to_browse() {
    let mut app = App::new();
    app.set_results(vec![make_result("Rust", "https://rust-lang.org")]);
    assert!(matches!(app.mode, Mode::Browse));
}

#[test]
fn set_results_selects_first_item() {
    let mut app = App::new();
    app.set_results(vec![
        make_result("First", "https://first.com"),
        make_result("Second", "https://second.com"),
    ]);
    assert_eq!(app.list_state.selected(), Some(0));
}

#[test]
fn set_results_empty_no_selection() {
    let mut app = App::new();
    app.set_results(vec![]);
    assert!(app.list_state.selected().is_none());
}

#[test]
fn set_results_stores_all_results() {
    let mut app = App::new();
    let results = vec![
        make_result("A", "https://a.com"),
        make_result("B", "https://b.com"),
        make_result("C", "https://c.com"),
    ];
    app.set_results(results);
    assert_eq!(app.results.len(), 3);
}

#[test]
fn set_results_replaces_previous_results() {
    let mut app = App::new();
    app.set_results(vec![make_result("Old", "https://old.com")]);
    app.set_results(vec![
        make_result("New1", "https://new1.com"),
        make_result("New2", "https://new2.com"),
    ]);
    assert_eq!(app.results.len(), 2);
    assert_eq!(app.results[0].title, "New1");
}

// ---------------------------------------------------------------------------
// App::next / App::prev
// ---------------------------------------------------------------------------

#[test]
fn next_advances_selection() {
    let mut app = App::new();
    app.set_results(vec![
        make_result("A", "https://a.com"),
        make_result("B", "https://b.com"),
        make_result("C", "https://c.com"),
    ]);
    app.next();
    assert_eq!(app.list_state.selected(), Some(1));
}

#[test]
fn next_clamps_at_last_item() {
    let mut app = App::new();
    app.set_results(vec![
        make_result("A", "https://a.com"),
        make_result("B", "https://b.com"),
    ]);
    app.next();
    app.next(); // already at last
    app.next();
    assert_eq!(app.list_state.selected(), Some(1));
}

#[test]
fn prev_retreats_selection() {
    let mut app = App::new();
    app.set_results(vec![
        make_result("A", "https://a.com"),
        make_result("B", "https://b.com"),
    ]);
    app.next();
    app.prev();
    assert_eq!(app.list_state.selected(), Some(0));
}

#[test]
fn prev_saturates_at_zero() {
    let mut app = App::new();
    app.set_results(vec![make_result("A", "https://a.com")]);
    app.prev();
    app.prev();
    assert_eq!(app.list_state.selected(), Some(0));
}

#[test]
fn next_is_noop_when_empty() {
    let mut app = App::new();
    app.set_results(vec![]);
    app.next();
    assert!(app.list_state.selected().is_none());
}

// ---------------------------------------------------------------------------
// App::selected_url
// ---------------------------------------------------------------------------

#[test]
fn selected_url_returns_url_of_selection() {
    let mut app = App::new();
    app.set_results(vec![
        make_result("First", "https://first.com"),
        make_result("Second", "https://second.com"),
    ]);
    assert_eq!(app.selected_url(), Some("https://first.com".to_string()));
    app.next();
    assert_eq!(app.selected_url(), Some("https://second.com".to_string()));
}

#[test]
fn selected_url_none_when_no_results() {
    let app = App::new();
    assert!(app.selected_url().is_none());
}

#[test]
fn selected_url_none_when_empty_results() {
    let mut app = App::new();
    app.set_results(vec![]);
    assert!(app.selected_url().is_none());
}

// ---------------------------------------------------------------------------
// Mode transitions via direct assignment (used by event loop)
// ---------------------------------------------------------------------------

#[test]
fn mode_can_transition_to_loading() {
    let mut app = App::new();
    app.mode = Mode::Loading;
    assert!(matches!(app.mode, Mode::Loading));
}

#[test]
fn mode_can_transition_to_error() {
    let mut app = App::new();
    app.mode = Mode::Error("something went wrong".to_string());
    if let Mode::Error(msg) = &app.mode {
        assert_eq!(msg, "something went wrong");
    } else {
        panic!("expected Error mode");
    }
}

#[test]
fn snippet_is_preserved_on_results() {
    let mut app = App::new();
    app.set_results(vec![make_result_with_snippet(
        "Rust docs",
        "https://doc.rust-lang.org",
        "The Rust programming language documentation.",
    )]);
    assert!(app.results[0].snippet.is_some());
}
