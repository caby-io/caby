use jiff::Timestamp;

pub mod token;

#[derive(Clone, Debug)]
pub struct Guest {
    pub id: String,
    pub created_at: Timestamp,
}
