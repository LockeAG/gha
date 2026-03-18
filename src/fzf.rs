use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Result};

use crate::github::GithubClient;
use crate::models::{Conclusion, RepoInfo, RunStatus, WorkflowRun};
use crate::ui::theme;

pub async fn pick_run(client: &GithubClient, repos: &[String], action: &str) -> Result<()> {
    if repos.is_empty() {
        bail!("No repos to search.");
    }

    let mut all_runs = Vec::new();
    for repo in repos {
        match client.fetch_runs(repo).await {
            Ok((resp, _)) => all_runs.extend(resp.workflow_runs),
            Err(_) => {}
        }
    }
    all_runs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    if all_runs.is_empty() {
        bail!("No workflow runs found.");
    }

    let lines: Vec<String> = all_runs
        .iter()
        .enumerate()
        .map(|(i, run)| format_run_line(i, run))
        .collect();

    if action == "detail" {
        // Loop: run picker ↔ detail view. Esc in detail goes back to runs.
        loop {
            let selection = match run_fzf(&lines, "workflow run") {
                Ok(s) => s,
                Err(_) => return Ok(()), // Esc at run list = exit
            };
            let run_idx = extract_index(&selection);
            if let Some(run) = run_idx.and_then(|i| all_runs.get(i)) {
                match show_detail(client, run).await {
                    Ok(()) => return Ok(()), // opened in browser, done
                    Err(_) => continue,      // Esc in detail = back to runs
                }
            }
        }
    }

    let selection = run_fzf(&lines, "workflow run")?;

    match action {
        "url" => {
            if let Some(u) = extract_url(&selection) {
                println!("{u}");
            }
        }
        "id" => {
            if let Some(num) = extract_run_number(&selection) {
                println!("{num}");
            }
        }
        _ => {
            if let Some(u) = extract_url(&selection) {
                open::that(u)?;
            }
        }
    }

    Ok(())
}

async fn show_detail(client: &GithubClient, run: &WorkflowRun) -> Result<()> {
    let repo = &run.repository.full_name;
    let (jobs_resp, _) = client.fetch_jobs(repo, run.id).await?;

    if jobs_resp.jobs.is_empty() {
        bail!("No jobs found for this run.");
    }

    let mut lines = Vec::new();
    for job in &jobs_resp.jobs {
        let icon = status_ansi(job.status, job.conclusion);
        let duration = format_duration(job.started_at, job.completed_at);
        lines.push(format!(
            "{icon} {name:<40} {dur:>10}  {url}",
            name = job.name,
            dur = duration,
            url = job.html_url,
        ));

        if let Some(steps) = &job.steps {
            let count = steps.len();
            for (si, step) in steps.iter().enumerate() {
                let s_icon = status_ansi(step.status, step.conclusion);
                let s_dur = format_duration(step.started_at, step.completed_at);
                let tree = if si == count - 1 {
                    "\x1b[90m\u{2514}\u{2500}\x1b[0m"
                } else {
                    "\x1b[90m\u{251C}\u{2500}\x1b[0m"
                };
                // Steps use the job URL since steps don't have their own
                lines.push(format!(
                    "   {tree} {s_icon} {name:<36} {dur:>10}  {url}",
                    name = step.name,
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

    let selection = run_fzf_with_header(&lines, &header)?;
    let url = extract_url(&selection);

    if let Some(u) = url {
        open::that(u)?;
    }

    Ok(())
}

pub async fn pick_repo(client: &GithubClient, orgs: &[String], action: &str) -> Result<()> {
    if orgs.is_empty() {
        bail!("No orgs specified. Use --org.");
    }

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
            let archived = if repo.archived {
                "\x1b[90m[archived]\x1b[0m "
            } else {
                ""
            };
            let age = repo
                .pushed_at
                .map(|t| format!("{} ago", theme::format_relative_time(t)))
                .unwrap_or_default();
            let desc = repo
                .description
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(50)
                .collect::<String>();
            format!(
                "{archived}{name:<35} {age:<10} {desc}",
                name = repo.full_name,
            )
        })
        .collect();

    let selection = run_fzf(&lines, "repo")?;

    let clean = strip_ansi(&selection);
    let repo_name = clean.split_whitespace().next().unwrap_or("").trim();

    match action {
        "name" => println!("{repo_name}"),
        _ => println!("{repo_name}"),
    }

    Ok(())
}

fn format_run_line(idx: usize, run: &WorkflowRun) -> String {
    let icon = status_ansi(run.status, run.conclusion);
    let repo = run
        .repository
        .full_name
        .split('/')
        .last()
        .unwrap_or(&run.repository.full_name);
    let name = run.name.as_deref().unwrap_or("\u{2014}");
    let branch = run.head_branch.as_deref().unwrap_or("\u{2014}");
    let age = theme::format_relative_time(run.updated_at);
    // Hidden index prefix for lookup after selection
    format!(
        "\x1b[0m\x1b[8m{idx}\x1b[0m {icon} {repo:<20} {name:<25} {branch:<15} {age:>5} #{:<6} {url}",
        run.run_number,
        url = run.html_url,
    )
}

fn run_fzf(lines: &[String], label: &str) -> Result<String> {
    run_fzf_with_header(lines, &format!("Select a {label}"))
}

fn run_fzf_with_header(lines: &[String], header: &str) -> Result<String> {
    let input = lines.join("\n");

    let mut child = Command::new("fzf")
        .args([
            "--ansi",
            "--no-sort",
            "--reverse",
            "--no-multi",
            &format!("--header={header}"),
            &format!("--color={}", theme::t().fzf_colors),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("fzf not found. Install fzf: https://github.com/junegunn/fzf")
            } else {
                e.into()
            }
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        bail!("fzf cancelled");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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

fn extract_url(line: &str) -> Option<&str> {
    line.split_whitespace()
        .rfind(|s| s.starts_with("https://"))
}

fn extract_run_number(line: &str) -> Option<&str> {
    line.split_whitespace()
        .find(|s| s.starts_with('#'))
        .map(|s| s.trim_start_matches('#'))
}

fn extract_index(line: &str) -> Option<usize> {
    // Hidden index is between ANSI invisible markers: \x1b[8m{idx}\x1b[0m
    let stripped = strip_ansi(line);
    stripped.split_whitespace().next()?.parse().ok()
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
