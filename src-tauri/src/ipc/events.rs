use serde::{Deserialize, Serialize};
use crate::tabs::tab::Tab;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabStateChangedPayload {
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationEventPayload {
    pub tab_id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabLoadingPayload {
    pub tab_id: String,
    pub url: String,
    pub is_loading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabTitlePayload {
    pub tab_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldStatsPayload {
    pub tab_id: String,
    pub blocked_trackers_count: u32,
    pub blocked_ads_count: u32,
    pub https_upgrades_count: u32,
    pub shields_enabled: bool,
}
