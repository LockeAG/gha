use std::collections::HashSet;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;

use crate::github::RateLimit;
use crate::models::{Conclusion, Job, RepoInfo, RunStatus, WorkflowRun};

const MAX_SEARCH_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Detail,
    RepoPicker,
    LogView,
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
    pub error_at: Option<Instant>,
    pub should_quit: bool,
    pub repos: Vec<String>,
    pub spinner_frame: usize,
    pub loading: bool,
    pub has_in_progress: bool,
    pub visible_rows: usize,
    // Repo picker
    pub all_org_repos: Vec<RepoInfo>,
    pub explicit_repos: Vec<String>,
    pub watched_set: HashSet<String>,
    pub picker_state: TableState,
    // Log viewer
    pub log_lines: Vec<String>,
    pub log_scroll: usize,
    pub log_job_name: String,
    pub log_group_positions: Vec<usize>,
}

impl App {
    pub fn new(
        watched: Vec<String>,
        explicit: Vec<String>,
        all_org_repos: Vec<RepoInfo>,
    ) -> Self {
        let watched_set: HashSet<String> = watched.iter().cloned().collect();
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
            error_at: None,
            should_quit: false,
            repos: watched,
            spinner_frame: 0,
            loading: true,
            has_in_progress: false,
            visible_rows: 20,
            all_org_repos,
            explicit_repos: explicit,
            watched_set,
            picker_state: TableState::default().with_selected(0),
            log_lines: Vec::new(),
            log_scroll: 0,
            log_job_name: String::new(),
            log_group_positions: Vec::new(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return None;
        }
        match self.view {
            View::RepoPicker => return self.handle_picker_key(key),
            View::LogView => return self.handle_log_key(key),
            _ => {}
        }
        match self.input_mode {
            InputMode::Search => self.handle_search_key(key),
            InputMode::Filter => self.handle_filter_key(key),
            InputMode::Normal => match self.view {
                View::Dashboard => self.handle_dashboard_key(key),
                View::Detail => self.handle_detail_key(key),
                View::RepoPicker | View::LogView => unreachable!(),
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
                if self.search_query.len() < MAX_SEARCH_LEN {
                    self.search_query.push(c);
                    self.apply_filters();
                }
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
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(self.half_page() as isize);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(-(self.half_page() as isize));
            }
            KeyCode::PageDown => self.move_selection(self.half_page() as isize),
            KeyCode::PageUp => self.move_selection(-(self.half_page() as isize)),
            KeyCode::Char('g') => self.first_run(),
            KeyCode::Char('G') => self.last_run(),
            KeyCode::Enter => return self.enter_detail(),
            KeyCode::Char('o') => return self.open_in_browser(),
            KeyCode::Char('/') => self.input_mode = InputMode::Search,
            KeyCode::Char('f') => self.input_mode = InputMode::Filter,
            KeyCode::Char('r') => return Some(AppAction::ForceRefresh),
            KeyCode::Char('R') => return self.rerun_selected(),
            KeyCode::Char('C') => return self.cancel_selected(),
            KeyCode::Char('a') => {
                if !self.all_org_repos.is_empty() {
                    self.view = View::RepoPicker;
                    self.picker_state.select(Some(0));
                }
            }
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
        let len = self.detail_row_count();
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => {
                self.view = View::Dashboard;
                self.jobs.clear();
                self.current_run_id = None;
                self.current_run_repo = None;
            }
            KeyCode::Char('j') | KeyCode::Down => {
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
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if len > 0 {
                    let jump = self.half_page();
                    let i = self
                        .detail_state
                        .selected()
                        .map_or(0, |i| (i + jump).min(len - 1));
                    self.detail_state.select(Some(i));
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let jump = self.half_page();
                let i = self
                    .detail_state
                    .selected()
                    .map_or(0, |i| i.saturating_sub(jump));
                self.detail_state.select(Some(i));
            }
            KeyCode::PageDown => {
                if len > 0 {
                    let jump = self.half_page();
                    let i = self
                        .detail_state
                        .selected()
                        .map_or(0, |i| (i + jump).min(len - 1));
                    self.detail_state.select(Some(i));
                }
            }
            KeyCode::PageUp => {
                let jump = self.half_page();
                let i = self
                    .detail_state
                    .selected()
                    .map_or(0, |i| i.saturating_sub(jump));
                self.detail_state.select(Some(i));
            }
            KeyCode::Char('g') => self.detail_state.select(Some(0)),
            KeyCode::Char('G') => {
                if len > 0 {
                    self.detail_state.select(Some(len - 1));
                }
            }
            KeyCode::Enter | KeyCode::Char('o') => return self.open_in_browser(),
            KeyCode::Char('R') => return self.rerun_current(),
            KeyCode::Char('C') => return self.cancel_current(),
            KeyCode::Char('L') | KeyCode::Char('l') => return self.view_logs(),
            _ => {}
        }
        None
    }

    fn handle_log_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        let max_scroll = self.log_lines.len().saturating_sub(self.visible_rows);
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => {
                self.view = View::Detail;
                self.log_lines.clear();
                self.log_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.log_scroll = (self.log_scroll + 1).min(max_scroll);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let jump = self.half_page();
                self.log_scroll = (self.log_scroll + jump).min(max_scroll);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let jump = self.half_page();
                self.log_scroll = self.log_scroll.saturating_sub(jump);
            }
            KeyCode::PageDown => {
                let jump = self.half_page();
                self.log_scroll = (self.log_scroll + jump).min(max_scroll);
            }
            KeyCode::PageUp => {
                let jump = self.half_page();
                self.log_scroll = self.log_scroll.saturating_sub(jump);
            }
            KeyCode::Char('g') => self.log_scroll = 0,
            KeyCode::Char('G') => self.log_scroll = max_scroll,
            KeyCode::Char('n') => {
                if let Some(&pos) = self
                    .log_group_positions
                    .iter()
                    .find(|&&p| p > self.log_scroll)
                {
                    self.log_scroll = pos.min(max_scroll);
                }
            }
            KeyCode::Char('N') => {
                if let Some(&pos) = self
                    .log_group_positions
                    .iter()
                    .rev()
                    .find(|&&p| p < self.log_scroll)
                {
                    self.log_scroll = pos;
                }
            }
            _ => {}
        }
        None
    }

    fn handle_picker_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        let len = self.all_org_repos.len();
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc | KeyCode::Char('a') => {
                // Apply changes and go back
                self.rebuild_repos();
                self.view = View::Dashboard;
                return Some(AppAction::ReposChanged(self.repos.clone()));
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if len > 0 {
                    let i = self
                        .picker_state
                        .selected()
                        .map_or(0, |i| (i + 1).min(len - 1));
                    self.picker_state.select(Some(i));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self
                    .picker_state
                    .selected()
                    .map_or(0, |i| i.saturating_sub(1));
                self.picker_state.select(Some(i));
            }
            KeyCode::Char('g') => self.picker_state.select(Some(0)),
            KeyCode::Char('G') => {
                if len > 0 {
                    self.picker_state.select(Some(len - 1));
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(idx) = self.picker_state.selected() {
                    if let Some(repo) = self.all_org_repos.get(idx) {
                        let name = repo.full_name.clone();
                        // Don't allow unchecking explicit repos
                        if !self.explicit_repos.contains(&name) {
                            if self.watched_set.contains(&name) {
                                self.watched_set.remove(&name);
                            } else {
                                self.watched_set.insert(name);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn rebuild_repos(&mut self) {
        let mut repos: Vec<String> = self.explicit_repos.clone();
        for r in &self.all_org_repos {
            if self.watched_set.contains(&r.full_name)
                && !repos.contains(&r.full_name)
            {
                repos.push(r.full_name.clone());
            }
        }
        self.repos = repos;
    }

    fn half_page(&self) -> usize {
        (self.visible_rows / 2).max(1)
    }

    fn detail_row_count(&self) -> usize {
        self.jobs
            .iter()
            .map(|j| 1 + j.steps.as_ref().map_or(0, |s| s.len()))
            .sum()
    }

    fn move_selection(&mut self, delta: isize) {
        if self.filtered_runs.is_empty() {
            return;
        }
        let max = self.filtered_runs.len() - 1;
        let current = self.table_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, max as isize) as usize;
        self.table_state.select(Some(next));
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
            _ => None,
        }
    }

    fn selected_job(&self) -> Option<&Job> {
        let row = self.detail_state.selected()?;
        let mut idx = 0;
        for job in &self.jobs {
            if idx == row {
                return Some(job);
            }
            idx += 1;
            let step_count = job.steps.as_ref().map_or(0, |s| s.len());
            if row < idx + step_count {
                // Selected a step — return its parent job
                return Some(job);
            }
            idx += step_count;
        }
        None
    }

    fn view_logs(&mut self) -> Option<AppAction> {
        let job = self.selected_job()?;
        if job.status != RunStatus::Completed {
            self.set_error("Logs only available for completed jobs".to_string());
            return None;
        }
        let repo = self.current_run_repo.clone()?;
        let job_id = job.id;
        let job_name = job.name.clone();
        self.log_lines.clear();
        self.log_scroll = 0;
        self.log_job_name = job_name;
        self.view = View::LogView;
        Some(AppAction::FetchLogs(repo, job_id))
    }

    fn rerun_selected(&self) -> Option<AppAction> {
        let run = self.selected_run()?;
        let repo = run.repository.full_name.clone();
        let run_id = run.id;
        if run.conclusion == Some(Conclusion::Failure) {
            Some(AppAction::RerunFailed(repo, run_id))
        } else {
            Some(AppAction::RerunWorkflow(repo, run_id))
        }
    }

    fn rerun_current(&self) -> Option<AppAction> {
        let run_id = self.current_run_id?;
        let run = self.runs.iter().find(|r| r.id == run_id)?;
        let repo = run.repository.full_name.clone();
        if run.conclusion == Some(Conclusion::Failure) {
            Some(AppAction::RerunFailed(repo, run_id))
        } else {
            Some(AppAction::RerunWorkflow(repo, run_id))
        }
    }

    fn cancel_selected(&self) -> Option<AppAction> {
        let run = self.selected_run()?;
        let repo = run.repository.full_name.clone();
        Some(AppAction::CancelWorkflow(repo, run.id))
    }

    fn cancel_current(&self) -> Option<AppAction> {
        let run_id = self.current_run_id?;
        let run = self.runs.iter().find(|r| r.id == run_id)?;
        let repo = run.repository.full_name.clone();
        Some(AppAction::CancelWorkflow(repo, run_id))
    }

    pub fn update_logs(&mut self, text: String) {
        self.log_lines = text.lines().map(String::from).collect();
        self.log_group_positions = self
            .log_lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.starts_with("##[group]"))
            .map(|(i, _)| i)
            .collect();
        // Auto-scroll to end (tail behavior)
        let max = self.log_lines.len().saturating_sub(self.visible_rows);
        self.log_scroll = max;
    }

    pub fn update_runs(&mut self, runs: Vec<WorkflowRun>, rate_limit: RateLimit) {
        let selected_run_id = self.selected_run().map(|r| r.id);

        self.runs = runs;
        self.runs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        self.rate_limit = Some(rate_limit);
        self.last_refresh = Some(chrono::Utc::now());
        self.loading = false;
        self.error = None;
        self.error_at = None;

        self.has_in_progress = self.runs.iter().any(|r| {
            matches!(
                r.status,
                RunStatus::InProgress | RunStatus::Queued | RunStatus::Waiting
            )
        });

        self.apply_filters();

        if let Some(target_id) = selected_run_id {
            if let Some(pos) = self
                .filtered_runs
                .iter()
                .position(|&idx| self.runs[idx].id == target_id)
            {
                self.table_state.select(Some(pos));
            }
        }
    }

    pub fn update_jobs(&mut self, run_id: u64, jobs: Vec<Job>) {
        if self.current_run_id == Some(run_id) && self.view == View::Detail {
            self.jobs = jobs;
        }
    }

    pub fn on_tick(&mut self) {
        if self.has_in_progress || self.loading {
            self.spinner_frame = (self.spinner_frame + 1) % 10;
        }

        if let Some(at) = self.error_at {
            if at.elapsed().as_secs() >= 8 {
                self.error = None;
                self.error_at = None;
            }
        }
    }

    pub fn set_error(&mut self, err: String) {
        let truncated = if err.len() > 60 {
            format!("{}...", &err[..57])
        } else {
            err
        };
        self.error = Some(truncated);
        self.error_at = Some(Instant::now());
    }

    pub fn mark_loading_done(&mut self) {
        self.loading = false;
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
    ReposChanged(Vec<String>),
    RerunWorkflow(String, u64),
    RerunFailed(String, u64),
    CancelWorkflow(String, u64),
    FetchLogs(String, u64),
}
