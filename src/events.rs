use crossterm::event::KeyEvent;

use crate::github::RateLimit;
use crate::models::{Job, WorkflowRun};

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    RunsUpdated(Vec<WorkflowRun>, RateLimit),
    JobsUpdated(u64, Vec<Job>),
    ApiError(String),
}
