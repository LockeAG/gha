use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

use crate::models::{JobsResponse, RepoInfo, WorkflowRunsResponse};

pub struct RateLimit {
    pub remaining: u64,
    pub limit: u64,
}

pub struct GithubClient {
    client: reqwest::Client,
    per_page: u8,
}

impl GithubClient {
    pub fn new(token: &str, per_page: u8) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("gha-tui/0.1"));
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { client, per_page })
    }

    pub async fn fetch_runs(&self, repo: &str) -> Result<(WorkflowRunsResponse, RateLimit)> {
        let url = format!(
            "https://api.github.com/repos/{repo}/actions/runs?per_page={}",
            self.per_page
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("fetch runs")?;
        let rl = parse_rate_limit(&resp);
        let body = resp
            .error_for_status()?
            .json::<WorkflowRunsResponse>()
            .await?;
        Ok((body, rl))
    }

    pub async fn fetch_org_repos(&self, owner: &str) -> Result<Vec<RepoInfo>> {
        let mut all_repos = Vec::new();
        let mut page = 1u32;

        loop {
            let org_url = format!(
                "https://api.github.com/orgs/{owner}/repos?per_page=100&sort=pushed&page={page}"
            );
            let resp = self.client.get(&org_url).send().await?;

            if resp.status().is_success() {
                let repos: Vec<RepoInfo> = resp.json().await?;
                let count = repos.len();
                all_repos.extend(repos);
                if count < 100 || page >= 3 {
                    break;
                }
                page += 1;
                continue;
            }

            if page == 1 {
                // Try user endpoint as fallback
                let user_url = format!(
                    "https://api.github.com/users/{owner}/repos?per_page=100&sort=pushed&page=1"
                );
                let resp = self.client.get(&user_url).send().await?;
                if resp.status().is_success() {
                    let repos: Vec<RepoInfo> = resp.json().await?;
                    let count = repos.len();
                    all_repos.extend(repos);
                    if count < 100 {
                        break;
                    }
                    // Fetch page 2 for users too
                    let user_url2 = format!(
                        "https://api.github.com/users/{owner}/repos?per_page=100&sort=pushed&page=2"
                    );
                    let resp2 = self.client.get(&user_url2).send().await?;
                    if resp2.status().is_success() {
                        let repos2: Vec<RepoInfo> = resp2.json().await?;
                        all_repos.extend(repos2);
                    }
                } else {
                    let status = resp.status();
                    anyhow::bail!("Failed to fetch repos for '{owner}': HTTP {status}");
                }
            }
            break;
        }

        Ok(all_repos)
    }

    pub async fn rerun_workflow(&self, repo: &str, run_id: u64) -> Result<()> {
        let url =
            format!("https://api.github.com/repos/{repo}/actions/runs/{run_id}/rerun");
        let resp = self
            .client
            .post(&url)
            .send()
            .await
            .context("rerun workflow")?;
        resp.error_for_status()?;
        Ok(())
    }

    pub async fn rerun_failed_jobs(&self, repo: &str, run_id: u64) -> Result<()> {
        let url =
            format!("https://api.github.com/repos/{repo}/actions/runs/{run_id}/rerun-failed-jobs");
        let resp = self
            .client
            .post(&url)
            .send()
            .await
            .context("rerun failed jobs")?;
        resp.error_for_status()?;
        Ok(())
    }

    pub async fn fetch_jobs(&self, repo: &str, run_id: u64) -> Result<(JobsResponse, RateLimit)> {
        let url =
            format!("https://api.github.com/repos/{repo}/actions/runs/{run_id}/jobs");
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("fetch jobs")?;
        let rl = parse_rate_limit(&resp);
        let body = resp.error_for_status()?.json::<JobsResponse>().await?;
        Ok((body, rl))
    }
}

fn parse_rate_limit(resp: &reqwest::Response) -> RateLimit {
    let h = resp.headers();
    let get = |name: &str| -> u64 {
        h.get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    RateLimit {
        remaining: get("x-ratelimit-remaining"),
        limit: get("x-ratelimit-limit"),
    }
}
