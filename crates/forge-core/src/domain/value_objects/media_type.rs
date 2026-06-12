use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediaOutputType {
    Image,
    Video,
    Audio,
    Design,
    Text,
    Code,
    Document,
    Presentation,
}
