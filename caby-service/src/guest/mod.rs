use jiff::Timestamp;

use crate::helpers::random_token;
pub mod token;

#[derive(Clone, Debug)]
pub struct Guest {
    pub id: String,
    pub created_at: Timestamp,
}

impl Guest {
    pub fn new() -> Self {
        Self {
            id: random_token(),
            created_at: Timestamp::now(),
        }
    }
}
