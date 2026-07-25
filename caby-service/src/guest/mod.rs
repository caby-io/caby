use chrono::{DateTime, Utc};

pub mod token;

#[derive(Clone, Debug)]
pub struct Guest {
    pub id: String,
    pub created_at: DateTime<Utc>,
}
