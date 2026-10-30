use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::privacy::filter::PrivacyEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteShieldSettings {
    pub shields_enabled: bool,
    pub block_scripts: bool,
    pub https_everywhere: bool,
}

impl Default for SiteShieldSettings {
    fn default() -> Self {
        Self {
            shields_enabled: true,
            block_scripts: false,
            https_everywhere: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabShieldStats {
    pub tab_id: String,
    pub blocked_trackers_count: u32,
    pub blocked_ads_count: u32,
    pub https_upgrades_count: u32,
    pub shields_enabled: bool,
}

pub struct ShieldManager {
    pub engine: PrivacyEngine,
    tab_stats: HashMap<String, TabShieldStats>,
    domain_settings: HashMap<String, SiteShieldSettings>,
}

#[derive(Clone)]
pub struct ShieldManagerState(pub Arc<Mutex<ShieldManager>>);

impl Default for ShieldManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShieldManager {
    pub fn new() -> Self {
        Self {
            engine: PrivacyEngine::new(),
            tab_stats: HashMap::new(),
            domain_settings: HashMap::new(),
        }
    }

    pub fn get_or_create_stats(&mut self, tab_id: &str) -> TabShieldStats {
        self.tab_stats
            .entry(tab_id.to_string())
            .or_insert_with(|| TabShieldStats {
                tab_id: tab_id.to_string(),
                blocked_trackers_count: 0,
                blocked_ads_count: 0,
                https_upgrades_count: 0,
                shields_enabled: true,
            })
            .clone()
    }

    pub fn record_blocked_tracker(&mut self, tab_id: &str) {
        let stats = self
            .tab_stats
            .entry(tab_id.to_string())
            .or_insert_with(|| TabShieldStats {
                tab_id: tab_id.to_string(),
                blocked_trackers_count: 0,
                blocked_ads_count: 0,
                https_upgrades_count: 0,
                shields_enabled: true,
            });
        stats.blocked_trackers_count += 1;
    }

    pub fn record_blocked_ad(&mut self, tab_id: &str) {
        let stats = self
            .tab_stats
            .entry(tab_id.to_string())
            .or_insert_with(|| TabShieldStats {
                tab_id: tab_id.to_string(),
                blocked_trackers_count: 0,
                blocked_ads_count: 0,
                https_upgrades_count: 0,
                shields_enabled: true,
            });
        stats.blocked_ads_count += 1;
    }

    pub fn set_shields_enabled(&mut self, tab_id: &str, enabled: bool) {
        if let Some(stats) = self.tab_stats.get_mut(tab_id) {
            stats.shields_enabled = enabled;
        }
    }

    pub fn is_shield_enabled(&self, tab_id: &str) -> bool {
        self.tab_stats
            .get(tab_id)
            .map(|s| s.shields_enabled)
            .unwrap_or(true)
    }

    pub fn reset_tab_stats(&mut self, tab_id: &str) {
        if let Some(stats) = self.tab_stats.get_mut(tab_id) {
            stats.blocked_trackers_count = 0;
            stats.blocked_ads_count = 0;
            stats.https_upgrades_count = 0;
        }
    }
}

