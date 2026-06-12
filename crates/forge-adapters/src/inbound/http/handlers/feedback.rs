use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FeedbackRequest {
    pub input: String,
    pub rating: i8,
}
