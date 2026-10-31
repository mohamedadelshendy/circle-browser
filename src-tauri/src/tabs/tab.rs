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
        let is_nt = initial_url.is_empty()
            || initial_url == "about:blank"
            || initial_url.contains("newtab.html")
            || initial_url == "browser://newtab";
        let title = if is_nt {
            "New tab".to_string()
        } else {
            crate::engine::NavigationResolver::title_from_url(&initial_url)
        };
        let favicon = if is_nt {
            None
        } else if let Ok(parsed) = url::Url::parse(&initial_url) {
            parsed.host_str().map(|h| format!("https://www.google.com/s2/favicons?domain={h}&sz=32"))
        } else {
            None
        };
        Self {
            id,
            title,
            url: initial_url,
            favicon,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
        }
    }
}

