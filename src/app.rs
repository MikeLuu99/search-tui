use metadata_search_engine_rs::models::AggregatedResult;
use ratatui::widgets::ListState;

pub enum Mode {
    Input,
    Loading,
    Browse,
    Error(String),
}

/// A completed search fan-out: aggregated results plus which engines failed,
/// so the UI can surface partial failures instead of hiding them.
pub struct SearchOutcome {
    pub results: Vec<AggregatedResult>,
    pub engines_failed: Vec<String>,
}

pub struct App {
    pub mode: Mode,
    pub input: String,
    pub results: Vec<AggregatedResult>,
    pub engines_failed: Vec<String>,
    pub list_state: ListState,
}

// 1. Implementation of the Default trait
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// 2. Inherent implementation for App methods
impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Input,
            input: String::new(),
            results: Vec::new(),
            engines_failed: Vec::new(),
            list_state: ListState::default(),
        }
    }

    pub fn set_results(&mut self, results: Vec<AggregatedResult>) {
        self.results = results;
        self.list_state.select(if self.results.is_empty() {
            None
        } else {
            Some(0)
        });
        self.mode = Mode::Browse;
    }

    pub fn set_outcome(&mut self, outcome: SearchOutcome) {
        self.engines_failed = outcome.engines_failed;
        self.set_results(outcome.results);
    }

    pub fn next(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let next = self
            .list_state
            .selected()
            .map_or(0, |i| (i + 1).min(self.results.len() - 1));
        self.list_state.select(Some(next));
    }

    pub fn prev(&mut self) {
        let prev = self
            .list_state
            .selected()
            .map_or(0, |i| i.saturating_sub(1));
        self.list_state.select(Some(prev));
    }

    pub fn selected_url(&self) -> Option<String> {
        self.list_state
            .selected()
            .and_then(|i| self.results.get(i))
            .map(|r| r.url.clone())
    }
}
