use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Result};

use crate::github::GithubClient;
use crate::models::{Conclusion, RepoInfo, RunStatus};
use crate::ui::theme;

pub async fn pick_run(client: &GithubClient, repos: &[String], action: &str) -> Result<()> {
    if repos.is_empty() {
        bail!("No repos to search.");
    }

    let mut all_runs = Vec::new();
    for repo in repos {
        match client.fetch_runs(repo).await {
            Ok((resp, _)) => all_runs.extend(resp.workflow_runs),
            Err(_) => {} // skip failures silently for fzf mode
        }
    }
    all_runs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    if all_runs.is_empty() {
        bail!("No workflow runs found.");
    }

    let lines: Vec<String> = all_runs
        .iter()
        .map(|run| {
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
            format!(
                "{icon} {repo:<20} {name:<25} {branch:<15} {age:>5} #{:<6} {url}",
                run.run_number,
                url = run.html_url,
            )
        })
        .collect();

    let selection = run_fzf(&lines, "workflow run")?;
    let url = extract_url(&selection);

    match action {
        "url" => {
            if let Some(u) = url {
                println!("{u}");
            }
        }
        "id" => {
            // Extract run number from selection
            if let Some(num) = extract_run_number(&selection) {
                println!("{num}");
            }
        }
        _ => {
            // "open" (default)
            if let Some(u) = url {
                open::that(u)?;
            }
        }
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

    // Extract repo name (first non-ansi field)
    let clean = strip_ansi(&selection);
    let repo_name = clean.split_whitespace().next().unwrap_or("").trim();

    match action {
        "name" => println!("{repo_name}"),
        _ => {
            // Default: print for piping
            println!("{repo_name}");
        }
    }

    Ok(())
}

fn run_fzf(lines: &[String], label: &str) -> Result<String> {
    let input = lines.join("\n");

    let mut child = Command::new("fzf")
        .args([
            "--ansi",
            "--no-sort",
            "--reverse",
            "--no-multi",
            &format!("--header=Select a {label}"),
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

fn extract_url(line: &str) -> Option<&str> {
    line.split_whitespace()
        .rfind(|s| s.starts_with("https://"))
}

fn extract_run_number(line: &str) -> Option<&str> {
    line.split_whitespace()
        .find(|s| s.starts_with('#'))
        .map(|s| s.trim_start_matches('#'))
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
