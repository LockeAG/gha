use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRunsResponse {
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: Option<String>,
    pub head_branch: Option<String>,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub html_url: String,
    pub run_number: u64,
    pub event: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub actor: Actor,
    pub repository: Repository,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Actor {
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub full_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    InProgress,
    Completed,
    Waiting,
    Requested,
    Pending,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
    Neutral,
    Stale,
    StartupFailure,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoInfo {
    pub full_name: String,
    pub pushed_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    #[serde(default)]
    pub archived: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JobsResponse {
    pub jobs: Vec<Job>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Job {
    pub id: u64,
    pub run_id: u64,
    pub name: String,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub html_url: String,
    pub steps: Option<Vec<Step>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Step {
    pub name: String,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub number: u64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}
