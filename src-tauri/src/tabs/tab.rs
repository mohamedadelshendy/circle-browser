use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

impl Tab {
    pub fn new(id: String, initial_url: String) -> Self {
        Self {
            id,
            title: "New tab".to_string(),
            url: initial_url,
            favicon: None,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
        }
    }
}

