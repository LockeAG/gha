use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

use crate::models::{JobsResponse, WorkflowRunsResponse};

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

    pub async fn fetch_org_repos(&self, org: &str) -> Result<Vec<String>> {
        let url = format!("https://api.github.com/orgs/{org}/repos?per_page=100&sort=pushed");
        let resp = self.client.get(&url).send().await?;
        let repos: Vec<serde_json::Value> = resp.error_for_status()?.json().await?;
        Ok(repos
            .iter()
            .filter_map(|r| r["full_name"].as_str().map(String::from))
            .collect())
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
