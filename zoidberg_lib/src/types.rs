use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Update {
    pub id: i64,
    pub status: String,
}
