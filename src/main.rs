mod app;
// mod html2md;
mod ui;

use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;

use app::{App, Mode, SearchOutcome};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use metadata_search_engine_rs::{
    aggregator::{aggregate, query_all_engines},
    cache::EngineLimits,
    engines::{
        BraveEngine, DuckDuckGoEngine, SearchEngine, StartpageEngine, YahooEngine,
        build_http_client,
    },
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Event loop
// ---------------------------------------------------------------------------

async fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    rx: &mut mpsc::Receiver<Result<SearchOutcome, String>>,
    tx: mpsc::Sender<Result<SearchOutcome, String>>,
    engines: Vec<Arc<dyn SearchEngine>>,
    limits: Arc<EngineLimits>,
    max_results: usize,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if let Ok(msg) = rx.try_recv() {
            match msg {
                Ok(outcome) => app.set_outcome(outcome),
                Err(e) => app.mode = Mode::Error(e),
            }
        }

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(());
        }

        if matches!(app.mode, Mode::Input) {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => {
                    let query = app.input.trim().to_string();
                    if !query.is_empty() {
                        app.mode = Mode::Loading;
                        let tx = tx.clone();
                        let engines = engines.clone();
                        let limits = Arc::clone(&limits);
                        tokio::spawn(async move {
                            tx.send(run_search(&engines, &limits, &query, max_results).await)
                                .await
                                .ok();
                        });
                    }
                }
                KeyCode::Char(c) => app.input.push(c),
                KeyCode::Backspace => {
                    app.input.pop();
                }
                _ => {}
            }
        } else if matches!(app.mode, Mode::Browse) {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.prev(),
                KeyCode::Char('g') => app.list_state.select(Some(0)),
                KeyCode::Char('G') => {
                    let last = app.results.len().saturating_sub(1);
                    app.list_state.select(Some(last));
                }
                KeyCode::Enter | KeyCode::Char('l') => {
                    if let Some(url) = app.selected_url() {
                        let _ = open::that_detached(url);
                    }
                }
                KeyCode::Char('h') | KeyCode::Char('/') => {
                    app.input.clear();
                    app.mode = Mode::Input;
                }
                _ => {}
            }
        } else if matches!(app.mode, Mode::Error(_)) {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('h') | KeyCode::Char('/') => {
                    app.input.clear();
                    app.mode = Mode::Input;
                }
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Command-line interface
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "sui", version, about = "Terminal UI for the metadata search engine")]
struct Cli {
    /// Query to search immediately on startup. Multiple words are joined.
    query: Vec<String>,

    /// Number of results to request per engine (default: 10).
    #[arg(short, long)]
    max_results: Option<usize>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let initial_query = cli.query.join(" ");
    let max_results = cli.max_results.unwrap_or(10);

    let client = Arc::new(build_http_client()?);
    let engines: Vec<Arc<dyn SearchEngine>> = vec![
        Arc::new(DuckDuckGoEngine::new(Arc::clone(&client))),
        Arc::new(BraveEngine::new(Arc::clone(&client))),
        Arc::new(StartpageEngine::new(Arc::clone(&client))),
        Arc::new(YahooEngine::new(Arc::clone(&client))),
    ];
    let limits = Arc::new(EngineLimits::new(1, std::time::Duration::from_secs(5)));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let (tx, mut rx) = mpsc::channel::<Result<SearchOutcome, String>>(1);
    let mut app = App::new();

    let initial_query = initial_query.trim().to_string();
    if !initial_query.is_empty() {
        app.input = initial_query.clone();
        app.mode = Mode::Loading;
        let tx2 = tx.clone();
        let engines2 = engines.clone();
        let limits2 = Arc::clone(&limits);
        tokio::spawn(async move {
            let _ = tx2
                .send(run_search(&engines2, &limits2, &initial_query, max_results).await)
                .await;
        });
    }

    let result = run(&mut terminal, &mut app, &mut rx, tx, engines, limits, max_results).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

// ---------------------------------------------------------------------------
// Search helpers
// ---------------------------------------------------------------------------

/// Run one query against all engines and build a [`SearchOutcome`].
///
/// If every engine fails we return an error message naming them; otherwise we
/// return the aggregated results plus the list of engines that failed, so the
/// UI can show partial-failure warnings.
async fn run_search(
    engines: &[Arc<dyn SearchEngine>],
    limits: &EngineLimits,
    query: &str,
    max_results: usize,
) -> Result<SearchOutcome, String> {
    let (successes, failures) = query_all_engines(engines, limits, query, max_results).await;

    if successes.is_empty() {
        let failed = failures
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("All engines failed to respond ({failed})."));
    }

    let engines_failed = failures
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();

    Ok(SearchOutcome {
        results: aggregate(successes, max_results),
        engines_failed,
    })
}
