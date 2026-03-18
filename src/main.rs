use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::{mpsc, watch};

mod app;
mod config;
mod events;
mod fzf;
mod github;
mod models;
mod ui;

use app::{App, AppAction};
use config::Config;
use events::AppEvent;
use github::GithubClient;
use models::RepoInfo;

#[derive(Parser)]
#[command(name = "gha", about = "GitHub Actions TUI tracker")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(long, global = true, help = "Watch all repos in an org (repeatable)")]
    org: Vec<String>,

    #[arg(long, global = true, help = "Watch specific repo owner/name (repeatable)")]
    repo: Vec<String>,

    #[arg(long, global = true, help = "Max runs per repo")]
    per_page: Option<u8>,

    #[arg(long, global = true, help = "Only watch repos active in last N days (0 = all)")]
    days: Option<u64>,

    #[arg(long, global = true, help = "GitHub token (or GH_TOKEN/GITHUB_TOKEN env, or gh auth token)")]
    token: Option<String>,

    #[arg(long, help = "Poll interval in seconds (min 10)")]
    interval: Option<u64>,

    #[arg(long, global = true, help = "Color theme: catppuccin-mocha, tokyo-night, tokyo-night-storm")]
    theme: Option<String>,

    #[arg(long, global = true, help = "Max repos to auto-watch from orgs (default 5, 0 = all)")]
    max_repos: Option<usize>,
}

/// Resolved settings after merging CLI > config > defaults
struct Settings {
    orgs: Vec<String>,
    repos: Vec<String>,
    per_page: u8,
    days: u64,
    interval: u64,
    theme: String,
    token: Option<String>,
    max_repos: usize,
}

impl Settings {
    fn from(cli: &Cli, cfg: &Config) -> Self {
        let mut orgs = cli.org.clone();
        if orgs.is_empty() {
            orgs = cfg.orgs.clone();
        }

        let mut repos = cli.repo.clone();
        if repos.is_empty() {
            repos = cfg.repos.clone();
        }

        Self {
            orgs,
            repos,
            per_page: cli.per_page.or(cfg.per_page).unwrap_or(20),
            days: cli.days.or(cfg.days).unwrap_or(7),
            interval: cli.interval.or(cfg.interval).unwrap_or(30),
            theme: cli
                .theme
                .clone()
                .or_else(|| cfg.theme.clone())
                .unwrap_or_else(|| "catppuccin-mocha".to_string()),
            token: cli.token.clone(),
            max_repos: cli.max_repos.or(cfg.max_repos).unwrap_or(5),
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// fzf picker for tmux popups
    Fzf {
        #[command(subcommand)]
        mode: FzfMode,
    },
    /// Generate sample config at ~/.config/gha/config.toml
    Init,
}

#[derive(Subcommand)]
enum FzfMode {
    /// Pick a workflow run
    Runs {
        /// Action: open (default), detail, url, id
        #[arg(long, default_value = "open")]
        action: String,
    },
    /// Pick a repo
    Repos {
        /// Action on selection: name (default)
        #[arg(long, default_value = "name")]
        action: String,
    },
}

fn resolve_token(cli_token: Option<String>) -> Result<String> {
    if let Some(t) = cli_token {
        return Ok(t);
    }
    if let Ok(t) = std::env::var("GH_TOKEN") {
        return Ok(t);
    }
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        return Ok(t);
    }
    match std::process::Command::new("gh").args(["auth", "token"]).output() {
        Ok(output) if output.status.success() => {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if token.is_empty() {
                bail!("gh auth token returned empty. Run: gh auth login")
            }
            Ok(token)
        }
        Ok(_) => bail!(
            "GitHub CLI found but not authenticated.\n\
             Run:  gh auth login\n\
             Or set GH_TOKEN / GITHUB_TOKEN env var, or use --token flag."
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => bail!(
            "No GitHub token found.\n\
             Option 1: Install GitHub CLI and run: gh auth login\n\
             Option 2: Set GH_TOKEN or GITHUB_TOKEN env var\n\
             Option 3: Use --token flag"
        ),
        Err(e) => bail!("Failed to run gh auth token: {e}"),
    }
}

fn resolve_repo_from_git() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_repo_from_url(&url)
}

fn parse_repo_from_url(url: &str) -> Option<String> {
    let cleaned = url.trim_end_matches(".git");
    if let Some(path) = cleaned.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        if parts.len() >= 2 {
            return Some(format!("{}/{}", parts[0], parts[1]));
        }
    }
    if let Some(path) = cleaned.strip_prefix("git@github.com:") {
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        if parts.len() >= 2 {
            return Some(format!("{}/{}", parts[0], parts[1]));
        }
    }
    None
}

fn filter_active_repos(repos: &[RepoInfo], days: u64) -> Vec<String> {
    if days == 0 {
        return repos.iter().map(|r| r.full_name.clone()).collect();
    }
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    repos
        .iter()
        .filter(|r| !r.archived && r.pushed_at.map(|t| t > cutoff).unwrap_or(false))
        .map(|r| r.full_name.clone())
        .collect()
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original(info);
    }));
}

async fn resolve_repos(
    settings: &Settings,
    client: &GithubClient,
) -> Result<(Vec<String>, Vec<String>, Vec<RepoInfo>)> {
    let explicit_repos: Vec<String> = settings.repos.clone();

    // Fetch all orgs concurrently
    let org_futures: Vec<_> = settings
        .orgs
        .iter()
        .map(|org| client.fetch_org_repos(org))
        .collect();

    let org_results = futures::future::join_all(org_futures).await;

    let mut all_org_repos: Vec<RepoInfo> = Vec::new();
    for (i, result) in org_results.into_iter().enumerate() {
        match result {
            Ok(repos) => all_org_repos.extend(repos),
            Err(e) => bail!("Failed to fetch repos for org '{}': {e}", settings.orgs[i]),
        }
    }

    all_org_repos.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));

    let active_org_repos = filter_active_repos(&all_org_repos, settings.days);
    let mut watched: Vec<String> = explicit_repos.clone();
    for r in &active_org_repos {
        if !watched.contains(r) {
            // Cap at max_repos (0 = unlimited)
            if settings.max_repos > 0 && watched.len() >= settings.max_repos + explicit_repos.len() {
                break;
            }
            watched.push(r.clone());
        }
    }

    if watched.is_empty() && all_org_repos.is_empty() {
        if let Some(repo) = resolve_repo_from_git() {
            watched.push(repo);
        } else {
            bail!("No repos specified. Use --repo, --org, config file, or run from a git repo.");
        }
    }

    Ok((watched, explicit_repos, all_org_repos))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load();
    let settings = Settings::from(&cli, &cfg);

    // Handle init subcommand before anything else
    if let Some(Command::Init) = &cli.command {
        let path = config::config_path();
        if path.exists() {
            println!("Config already exists: {}", path.display());
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, config::generate_sample())?;
            println!("Created: {}", path.display());
        }
        return Ok(());
    }

    ui::theme::init(&settings.theme);

    let token = resolve_token(settings.token.clone())?;
    let client = GithubClient::new(&token, settings.per_page)?;

    match &cli.command {
        Some(Command::Fzf { mode }) => {
            let result = async {
                let (watched, _, _) = resolve_repos(&settings, &client).await?;
                match mode {
                    FzfMode::Runs { action } => {
                        fzf::pick_run(&client, &watched, action).await
                    }
                    FzfMode::Repos { action } => {
                        fzf::pick_repo(&client, &settings.orgs, action).await
                    }
                }
            }
            .await;

            if let Err(e) = result {
                let msg = e.to_string();
                // "fzf cancelled" is normal (user pressed Esc), exit silently
                if msg != "fzf cancelled" {
                    eprintln!("gha: {msg}");
                    // Pause so tmux popup doesn't close before you can read
                    eprintln!("\nPress Enter to close...");
                    let _ = std::io::stdin().read_line(&mut String::new());
                }
            }
        }
        Some(Command::Init) => unreachable!(),
        None => {
            let interval = settings.interval.max(10);

            if !settings.orgs.is_empty() {
                eprint!("\x1b[90m\u{28CB} Fetching repos...\x1b[0m");
            }

            let (watched, explicit_repos, all_org_repos) =
                resolve_repos(&settings, &client).await?;

            // Clear the loading message before entering TUI
            eprint!("\r\x1b[K");

            install_panic_hook();

            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let result = run_app(
                &mut terminal,
                watched,
                explicit_repos,
                all_org_repos,
                client,
                interval,
            )
            .await;

            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            result?;
        }
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    watched: Vec<String>,
    explicit: Vec<String>,
    all_org_repos: Vec<RepoInfo>,
    client: GithubClient,
    interval: u64,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);
    let mut app = App::new(watched.clone(), explicit, all_org_repos);
    let client = Arc::new(client);

    let (repos_tx, repos_rx) = watch::channel(watched);
    let (active_tx, active_rx) = watch::channel(false);

    let tx_input = tx.clone();
    tokio::task::spawn_blocking(move || loop {
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press
                    && tx_input.blocking_send(AppEvent::Key(key)).is_err()
                {
                    break;
                }
            }
        }
    });

    let tx_tick = tx.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_millis(250));
        loop {
            tick.tick().await;
            if tx_tick.send(AppEvent::Tick).await.is_err() {
                break;
            }
        }
    });

    let tx_poll = tx.clone();
    let client_poll = client.clone();
    let fast_interval = 5u64; // seconds when runs are in-progress
    tokio::spawn(async move {
        loop {
            let current_repos = repos_rx.borrow().clone();
            let mut all_runs = Vec::new();
            let mut last_rl = None;
            let mut had_error = false;

            for repo in &current_repos {
                match client_poll.fetch_runs(repo).await {
                    Ok((resp, rl)) => {
                        all_runs.extend(resp.workflow_runs);
                        last_rl = Some(rl);
                    }
                    Err(e) => {
                        had_error = true;
                        let _ = tx_poll
                            .send(AppEvent::ApiError(format!("{repo}: {e}")))
                            .await;
                    }
                }
            }

            if let Some(rl) = last_rl {
                let poll_dur = if rl.remaining < 100 {
                    Duration::from_secs(interval.max(60))
                } else if *active_rx.borrow() {
                    Duration::from_secs(fast_interval)
                } else {
                    Duration::from_secs(interval)
                };
                let _ = tx_poll.send(AppEvent::RunsUpdated(all_runs, rl)).await;
                tokio::time::sleep(poll_dur).await;
            } else if had_error {
                let _ = tx_poll.send(AppEvent::LoadingDone).await;
                tokio::time::sleep(Duration::from_secs(interval)).await;
            } else {
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        }
    });

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        if let Some(ev) = rx.recv().await {
            match ev {
                AppEvent::Key(key) => {
                    if let Some(action) = app.handle_key(key) {
                        handle_action(action, &client, &tx, &repos_tx, &mut app).await;
                    }
                }
                AppEvent::Tick => app.on_tick(),
                AppEvent::RunsUpdated(runs, rl) => {
                    app.update_runs(runs, rl);
                    let _ = active_tx.send(app.has_in_progress);
                }
                AppEvent::JobsUpdated(run_id, jobs) => app.update_jobs(run_id, jobs),
                AppEvent::ApiError(err) => app.set_error(err),
                AppEvent::LoadingDone => app.mark_loading_done(),
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

async fn handle_action(
    action: AppAction,
    client: &Arc<GithubClient>,
    tx: &mpsc::Sender<AppEvent>,
    repos_tx: &watch::Sender<Vec<String>>,
    app: &mut App,
) {
    match action {
        AppAction::ForceRefresh => {
            app.loading = true;
            let c = client.clone();
            let t = tx.clone();
            let r = app.repos.clone();
            tokio::spawn(async move {
                let mut all_runs = Vec::new();
                let mut last_rl = None;
                for repo in &r {
                    match c.fetch_runs(repo).await {
                        Ok((resp, rl)) => {
                            all_runs.extend(resp.workflow_runs);
                            last_rl = Some(rl);
                        }
                        Err(e) => {
                            let _ = t.send(AppEvent::ApiError(format!("{repo}: {e}"))).await;
                        }
                    }
                }
                if let Some(rl) = last_rl {
                    let _ = t.send(AppEvent::RunsUpdated(all_runs, rl)).await;
                }
            });
        }
        AppAction::FetchJobs(repo, run_id) => {
            let c = client.clone();
            let t = tx.clone();
            tokio::spawn(async move {
                match c.fetch_jobs(&repo, run_id).await {
                    Ok((resp, _rl)) => {
                        let _ = t.send(AppEvent::JobsUpdated(run_id, resp.jobs)).await;
                    }
                    Err(e) => {
                        let _ = t.send(AppEvent::ApiError(format!("jobs: {e}"))).await;
                    }
                }
            });
        }
        AppAction::OpenUrl(url) => {
            let _ = open::that(&url);
        }
        AppAction::ReposChanged(new_repos) => {
            let _ = repos_tx.send(new_repos);
            app.loading = true;
            let c = client.clone();
            let t = tx.clone();
            let r = app.repos.clone();
            tokio::spawn(async move {
                let mut all_runs = Vec::new();
                let mut last_rl = None;
                for repo in &r {
                    match c.fetch_runs(repo).await {
                        Ok((resp, rl)) => {
                            all_runs.extend(resp.workflow_runs);
                            last_rl = Some(rl);
                        }
                        Err(e) => {
                            let _ = t.send(AppEvent::ApiError(format!("{repo}: {e}"))).await;
                        }
                    }
                }
                if let Some(rl) = last_rl {
                    let _ = t.send(AppEvent::RunsUpdated(all_runs, rl)).await;
                } else {
                    let _ = t.send(AppEvent::LoadingDone).await;
                }
            });
        }
    }
}
