use std::fs;
use std::process::{Command, Stdio};

use anyhow::{bail, Result};

use crate::github::GithubClient;
use crate::models::{Conclusion, RepoInfo, RunStatus, WorkflowRun};
use crate::ui::theme;

// ANSI helpers
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[90m";
const BOLD: &str = "\x1b[1m";
const WHITE: &str = "\x1b[97m";

// Tab delimiter for fzf --delimiter
const TAB: char = '\t';

pub async fn pick_run(client: &GithubClient, repos: &[String], action: &str) -> Result<()> {
    if repos.is_empty() {
        bail!("No repos to search.");
    }

    eprint!(
        "\x1b[90mFetching {} repo{}...\x1b[0m\r",
        repos.len(),
        if repos.len() == 1 { "" } else { "s" }
    );

    let all_runs = fetch_all_runs(client, repos).await;

    if all_runs.is_empty() {
        bail!("No workflow runs found.");
    }

    // Format: icon \t repo \t workflow \t branch \t age \t #num \t URL
    let lines: Vec<String> = all_runs
        .iter()
        .enumerate()
        .map(|(i, run)| format_run_line(i, run))
        .collect();

    let header = format!(
        "  {DIM}Repo{RESET}{TAB}{DIM}Workflow{RESET}{TAB}{DIM}Branch{RESET}{TAB}{DIM}  Age{RESET}{TAB}{DIM}    #{RESET}"
    );

    if action == "detail" {
        loop {
            let selection = match run_fzf_tabbed(&lines, &header, 6) {
                Ok(s) => s,
                Err(_) => return Ok(()),
            };
            let run_idx = extract_field(&selection, 0)
                .and_then(|s| strip_ansi(&s).trim().parse::<usize>().ok());
            if let Some(run) = run_idx.and_then(|i| all_runs.get(i)) {
                match show_detail(client, run).await {
                    Ok(()) => return Ok(()),
                    Err(_) => continue,
                }
            }
        }
    }

    let selection = run_fzf_tabbed(&lines, &header, 6)?;

    match action {
        "url" => {
            if let Some(url) = extract_field(&selection, 6) {
                println!("{}", url.trim());
            }
        }
        "id" => {
            if let Some(num) = extract_field(&selection, 5) {
                println!("{}", strip_ansi(&num).trim().trim_start_matches('#'));
            }
        }
        _ => {
            if let Some(url) = extract_field(&selection, 6) {
                open::that(url.trim())?;
            }
        }
    }

    Ok(())
}

async fn show_detail(client: &GithubClient, run: &WorkflowRun) -> Result<()> {
    eprint!("\x1b[90mLoading jobs...\x1b[0m\r");
    let repo = &run.repository.full_name;
    let (jobs_resp, _) = client.fetch_jobs(repo, run.id).await?;

    if jobs_resp.jobs.is_empty() {
        bail!("No jobs found for this run.");
    }

    // Format: tree+icon \t name \t duration \t URL
    let mut lines = Vec::new();
    for job in &jobs_resp.jobs {
        let icon = status_ansi(job.status, job.conclusion);
        let dur = format_duration(job.started_at, job.completed_at);
        let jname = truncate(&job.name, 38);
        lines.push(format!(
            " {icon}{TAB}{BOLD}{WHITE}{jname}{RESET}{TAB}{DIM}{dur:>8}{RESET}{TAB}{url}",
            url = job.html_url,
        ));

        if let Some(steps) = &job.steps {
            let count = steps.len();
            for (si, step) in steps.iter().enumerate() {
                let s_icon = status_ansi(step.status, step.conclusion);
                let s_dur = format_duration(step.started_at, step.completed_at);
                let tree = if si == count - 1 {
                    "\u{2514}\u{2500}"
                } else {
                    "\u{251C}\u{2500}"
                };
                let sname = truncate(&step.name, 36);
                lines.push(format!(
                    " {DIM}{tree}{RESET} {s_icon}{TAB}{sname}{TAB}{DIM}{dur:>8}{RESET}{TAB}{url}",
                    dur = s_dur,
                    url = job.html_url,
                ));
            }
        }
    }

    let header = format!(
        "{} \u{2014} {} #{} \u{2502} {} \u{2502} {}",
        run.repository.full_name,
        run.name.as_deref().unwrap_or("workflow"),
        run.run_number,
        run.head_branch.as_deref().unwrap_or("\u{2014}"),
        run.actor.login,
    );

    let selection = run_fzf_tabbed(&lines, &header, 3)?;

    if let Some(url) = extract_field(&selection, 3) {
        open::that(url.trim())?;
    }

    Ok(())
}

pub async fn pick_repo(client: &GithubClient, orgs: &[String], action: &str) -> Result<()> {
    if orgs.is_empty() {
        bail!("No orgs specified. Use --org.");
    }

    eprint!("\x1b[90mFetching repos...\x1b[0m\r");

    let mut all_repos: Vec<RepoInfo> = Vec::new();
    for org in orgs {
        match client.fetch_org_repos(org).await {
            Ok(repos) => all_repos.extend(repos),
            Err(e) => bail!("Failed to fetch repos for '{org}': {e}"),
        }
    }
    all_repos.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));

    if all_repos.is_empty() {
        bail!("No repos found.");
    }

    let lines: Vec<String> = all_repos
        .iter()
        .map(|repo| {
            let age = repo
                .pushed_at
                .map(|t| theme::format_relative_time(t))
                .unwrap_or_default();
            let desc = repo
                .description
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(50)
                .collect::<String>();
            let tag = if repo.archived {
                format!(" {DIM}[archived]{RESET}")
            } else {
                String::new()
            };
            let rname = truncate(&repo.full_name, 30);
            format!(
                "{WHITE}{rname}{RESET}{tag}{TAB}{DIM}{age:>5} ago{RESET}{TAB}{DIM}{desc}{RESET}",
            )
        })
        .collect();

    let header = format!("{DIM}Repo{RESET}{TAB}{DIM}  Active{RESET}{TAB}{DIM}Description{RESET}");
    let selection = run_fzf_tabbed(&lines, &header, 3)?;

    let clean = strip_ansi(&selection);
    let repo_name = clean.split('\t').next().unwrap_or("").trim();

    println!("{repo_name}");

    Ok(())
}

fn format_run_line(idx: usize, run: &WorkflowRun) -> String {
    let icon = status_ansi(run.status, run.conclusion);
    let repo = truncate(
        run.repository
            .full_name
            .split('/')
            .last()
            .unwrap_or(&run.repository.full_name),
        18,
    );
    let name = truncate(run.name.as_deref().unwrap_or("\u{2014}"), 24);
    let branch = truncate(run.head_branch.as_deref().unwrap_or("\u{2014}"), 16);
    let age = theme::format_relative_time(run.updated_at);
    let num = format!("#{}", run.run_number);

    format!(
        "\x1b[8m{idx}\x1b[0m {icon}{TAB}{WHITE}{repo}{RESET}{TAB}{name}{TAB}{DIM}{branch}{RESET}{TAB}{DIM}{age:>5}{RESET}{TAB}{DIM}{num:>5}{RESET}{TAB}{url}",
        url = run.html_url,
    )
}

fn run_fzf_tabbed(lines: &[String], header: &str, hide_from: usize) -> Result<String> {
    let input = lines.join("\n");

    let tmp = std::env::temp_dir().join(format!("gha-fzf-{}", std::process::id()));
    fs::write(&tmp, &input)?;
    let input_file = fs::File::open(&tmp)?;

    let with_nth = format!("--with-nth=1..{hide_from}");

    let child = Command::new("fzf")
        .args([
            "--ansi",
            "--no-sort",
            "--reverse",
            "--no-multi",
            "--delimiter=\t",
            &with_nth,
            "--tabstop=18",
            &format!("--header={header}"),
            &format!("--color={}", theme::t().fzf_colors),
        ])
        .stdin(input_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            let _ = fs::remove_file(&tmp);
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("fzf not found. Install fzf: https://github.com/junegunn/fzf")
            } else {
                e.into()
            }
        })?;

    let output = child.wait_with_output()?;
    let _ = fs::remove_file(&tmp);

    if !output.status.success() {
        bail!("fzf cancelled");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn fetch_all_runs(client: &GithubClient, repos: &[String]) -> Vec<WorkflowRun> {
    let futures: Vec<_> = repos
        .iter()
        .map(|repo| client.fetch_runs(repo))
        .collect();

    let results = futures::future::join_all(futures).await;

    let mut all_runs: Vec<WorkflowRun> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .flat_map(|(resp, _)| resp.workflow_runs)
        .collect();

    all_runs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    all_runs
}

fn status_ansi(status: RunStatus, conclusion: Option<Conclusion>) -> &'static str {
    match status {
        RunStatus::Completed => match conclusion {
            Some(Conclusion::Success) => "\x1b[32m\u{2713}\x1b[0m",
            Some(Conclusion::Failure) => "\x1b[31m\u{2717}\x1b[0m",
            Some(Conclusion::Cancelled) => "\x1b[90m\u{2298}\x1b[0m",
            _ => "\x1b[90m?\x1b[0m",
        },
        RunStatus::InProgress => "\x1b[33m\u{25CF}\x1b[0m",
        RunStatus::Queued | RunStatus::Waiting | RunStatus::Pending | RunStatus::Requested => {
            "\x1b[34m\u{25CB}\x1b[0m"
        }
        RunStatus::Unknown => "\x1b[90m?\x1b[0m",
    }
}

fn format_duration(
    started: Option<chrono::DateTime<chrono::Utc>>,
    completed: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (started, completed) {
        (Some(start), Some(end)) => {
            let secs = (end - start).num_seconds().max(0);
            if secs < 60 {
                format!("{secs}s")
            } else if secs < 3600 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
            }
        }
        (Some(start), None) => {
            let secs = (chrono::Utc::now() - start).num_seconds().max(0);
            if secs < 60 {
                format!("{secs}s...")
            } else {
                format!("{}m...", secs / 60)
            }
        }
        _ => "\u{2014}".to_string(),
    }
}

fn extract_field<'a>(line: &'a str, idx: usize) -> Option<&'a str> {
    line.split('\t').nth(idx)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max.saturating_sub(1);
    // Don't split mid-char
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\u{2026}", &s[..end])
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}
