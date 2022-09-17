use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Update {
    pub worker: i32,
    pub job: i32,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Status {
    Submitted,
    Running,
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
            Status::Running => write!(f, "running"),
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
    pub id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Submit {
    pub cmd: String,
}
