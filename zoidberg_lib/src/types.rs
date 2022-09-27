use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Update {
    pub worker: String,
    pub job: i32,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Status {
    Submitted,
    Running(String),
    Completed,
    Failed,
}

impl Status {
    fn default() -> Self {
        Status::Submitted
    }
}

impl fmt::Display for Status {
    fn fmt(self: &Self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Submitted => write!(f, "submitted"),
            Status::Running(w) => write!(f, "running on worker {}", w),
            Status::Completed => write!(f, "completed"),
            Status::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatusRequest {
    pub id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    #[serde(default)]
    pub id: i32,
    pub cmd: String,
    #[serde(default = "Status::default")]
    pub status: Status,
}

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct FetchRequest {
    pub worker_id: String,
}

#[derive(Serialize, Deserialize)]
pub enum FetchResponse {
    Jobs(Vec<Job>),
    Terminate(String),
    Nop,
}

#[derive(Serialize, Deserialize)]
pub struct Submit {
    pub cmd: String,
}

#[derive(Serialize, Deserialize)]
pub struct Worker {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub last_heartbeat: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct Heartbeat {
    #[serde(default)]
    pub id: String,
}
