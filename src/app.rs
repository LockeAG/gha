use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;

use crate::github::RateLimit;
use crate::models::{Conclusion, Job, RunStatus, WorkflowRun};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickFilter {
    All,
    Failed,
    Running,
    Success,
}

pub struct App {
    pub runs: Vec<WorkflowRun>,
    pub filtered_runs: Vec<usize>,
    pub jobs: Vec<Job>,
    pub current_run_id: Option<u64>,
    pub current_run_repo: Option<String>,
    pub view: View,
    pub input_mode: InputMode,
    pub search_query: String,
    pub quick_filter: QuickFilter,
    pub table_state: TableState,
    pub detail_state: TableState,
    pub rate_limit: Option<RateLimit>,
    pub last_refresh: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
    pub should_quit: bool,
    pub repos: Vec<String>,
    pub spinner_frame: usize,
    pub loading: bool,
}

impl App {
    pub fn new(repos: Vec<String>, _poll_interval: u64) -> Self {
        Self {
            runs: Vec::new(),
            filtered_runs: Vec::new(),
            jobs: Vec::new(),
            current_run_id: None,
            current_run_repo: None,
            view: View::Dashboard,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            quick_filter: QuickFilter::All,
            table_state: TableState::default().with_selected(0),
            detail_state: TableState::default().with_selected(0),
            rate_limit: None,
            last_refresh: None,
            error: None,
            should_quit: false,
            repos,
            spinner_frame: 0,
            loading: true,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return None;
        }
        match self.input_mode {
            InputMode::Search => self.handle_search_key(key),
            InputMode::Filter => self.handle_filter_key(key),
            InputMode::Normal => match self.view {
                View::Dashboard => self.handle_dashboard_key(key),
                View::Detail => self.handle_detail_key(key),
            },
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
                self.apply_filters();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_filters();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_filters();
            }
            _ => {}
        }
        None
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('f') => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('1') => {
                self.quick_filter = QuickFilter::All;
                self.apply_filters();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('2') => {
                self.quick_filter = QuickFilter::Failed;
                self.apply_filters();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('3') => {
                self.quick_filter = QuickFilter::Running;
                self.apply_filters();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('4') => {
                self.quick_filter = QuickFilter::Success;
                self.apply_filters();
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        None
    }

    fn handle_dashboard_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.next_run(),
            KeyCode::Char('k') | KeyCode::Up => self.prev_run(),
            KeyCode::Char('g') => self.first_run(),
            KeyCode::Char('G') => self.last_run(),
            KeyCode::Enter => return self.enter_detail(),
            KeyCode::Char('o') => return self.open_in_browser(),
            KeyCode::Char('/') => self.input_mode = InputMode::Search,
            KeyCode::Char('f') => self.input_mode = InputMode::Filter,
            KeyCode::Char('r') => return Some(AppAction::ForceRefresh),
            KeyCode::Char('1') => {
                self.quick_filter = QuickFilter::All;
                self.apply_filters();
            }
            KeyCode::Char('2') => {
                self.quick_filter = QuickFilter::Failed;
                self.apply_filters();
            }
            KeyCode::Char('3') => {
                self.quick_filter = QuickFilter::Running;
                self.apply_filters();
            }
            KeyCode::Char('4') => {
                self.quick_filter = QuickFilter::Success;
                self.apply_filters();
            }
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.apply_filters();
                }
            }
            _ => {}
        }
        None
    }

    fn handle_detail_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => {
                self.view = View::Dashboard;
                self.jobs.clear();
                self.current_run_id = None;
                self.current_run_repo = None;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.detail_row_count();
                if len > 0 {
                    let i = self
                        .detail_state
                        .selected()
                        .map_or(0, |i| (i + 1).min(len - 1));
                    self.detail_state.select(Some(i));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self
                    .detail_state
                    .selected()
                    .map_or(0, |i| i.saturating_sub(1));
                self.detail_state.select(Some(i));
            }
            KeyCode::Char('g') => self.detail_state.select(Some(0)),
            KeyCode::Char('G') => {
                let len = self.detail_row_count();
                if len > 0 {
                    self.detail_state.select(Some(len - 1));
                }
            }
            KeyCode::Enter | KeyCode::Char('o') => return self.open_in_browser(),
            _ => {}
        }
        None
    }

    fn detail_row_count(&self) -> usize {
        self.jobs
            .iter()
            .map(|j| 1 + j.steps.as_ref().map_or(0, |s| s.len()))
            .sum()
    }

    fn next_run(&mut self) {
        if self.filtered_runs.is_empty() {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| (i + 1).min(self.filtered_runs.len() - 1));
        self.table_state.select(Some(i));
    }

    fn prev_run(&mut self) {
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| i.saturating_sub(1));
        self.table_state.select(Some(i));
    }

    fn first_run(&mut self) {
        if !self.filtered_runs.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    fn last_run(&mut self) {
        if !self.filtered_runs.is_empty() {
            self.table_state.select(Some(self.filtered_runs.len() - 1));
        }
    }

    pub fn selected_run(&self) -> Option<&WorkflowRun> {
        let idx = self.table_state.selected()?;
        let run_idx = self.filtered_runs.get(idx)?;
        self.runs.get(*run_idx)
    }

    fn enter_detail(&mut self) -> Option<AppAction> {
        let run = self.selected_run()?;
        let run_id = run.id;
        let repo = run.repository.full_name.clone();
        self.current_run_id = Some(run_id);
        self.current_run_repo = Some(repo.clone());
        self.view = View::Detail;
        self.detail_state.select(Some(0));
        self.jobs.clear();
        Some(AppAction::FetchJobs(repo, run_id))
    }

    fn open_in_browser(&self) -> Option<AppAction> {
        match self.view {
            View::Dashboard => {
                let run = self.selected_run()?;
                Some(AppAction::OpenUrl(run.html_url.clone()))
            }
            View::Detail => {
                let run_id = self.current_run_id?;
                let run = self.runs.iter().find(|r| r.id == run_id)?;
                Some(AppAction::OpenUrl(run.html_url.clone()))
            }
        }
    }

    pub fn update_runs(&mut self, runs: Vec<WorkflowRun>, rate_limit: RateLimit) {
        self.runs = runs;
        self.runs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        self.rate_limit = Some(rate_limit);
        self.last_refresh = Some(chrono::Utc::now());
        self.loading = false;
        self.error = None;
        self.apply_filters();
    }

    pub fn update_jobs(&mut self, _run_id: u64, jobs: Vec<Job>) {
        self.jobs = jobs;
    }

    pub fn on_tick(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 10;
    }

    pub fn apply_filters(&mut self) {
        self.filtered_runs = (0..self.runs.len())
            .filter(|&i| {
                let run = &self.runs[i];
                let passes_filter = match self.quick_filter {
                    QuickFilter::All => true,
                    QuickFilter::Failed => run.conclusion == Some(Conclusion::Failure),
                    QuickFilter::Running => matches!(
                        run.status,
                        RunStatus::InProgress | RunStatus::Queued | RunStatus::Waiting
                    ),
                    QuickFilter::Success => run.conclusion == Some(Conclusion::Success),
                };
                let passes_search = if self.search_query.is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    run.repository.full_name.to_lowercase().contains(&q)
                        || run
                            .name
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&q)
                        || run
                            .head_branch
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&q)
                        || run.actor.login.to_lowercase().contains(&q)
                        || run.event.to_lowercase().contains(&q)
                };
                passes_filter && passes_search
            })
            .collect();

        if let Some(sel) = self.table_state.selected() {
            if sel >= self.filtered_runs.len() {
                self.table_state.select(if self.filtered_runs.is_empty() {
                    None
                } else {
                    Some(self.filtered_runs.len() - 1)
                });
            }
        } else if !self.filtered_runs.is_empty() {
            self.table_state.select(Some(0));
        }
    }

}

pub enum AppAction {
    ForceRefresh,
    FetchJobs(String, u64),
    OpenUrl(String),
}
