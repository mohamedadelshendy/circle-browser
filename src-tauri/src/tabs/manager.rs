use std::sync::Arc;
use parking_lot::Mutex;
use tauri::{
    webview::{PageLoadEvent, WebviewBuilder},
    AppHandle, Emitter, Manager, Url, WebviewUrl, Window,
};
use uuid::Uuid;

use crate::engine::{BoundsCalculator, NavigationResolver};
use crate::ipc::events::{
    NavigationEventPayload, ShieldStatsPayload, TabLoadingPayload, TabStateChangedPayload,
};
use crate::privacy::{CosmeticFilter, ShieldManager};
use crate::tabs::tab::Tab;

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<String>,
    pub shield_manager: Arc<Mutex<ShieldManager>>,
}

#[derive(Clone)]
pub struct TabManagerState(pub Arc<Mutex<TabManager>>);

impl TabManager {
    pub fn new(shield_manager: Arc<Mutex<ShieldManager>>) -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            shield_manager,
        }
    }

    /// Spawns a new native child WebView, injects privacy defusers, and positions it beneath the chrome bar.
    pub fn create_tab(
        &mut self,
        app_handle: &AppHandle,
        window: &Window,
        target_url: Option<String>,
    ) -> Result<Tab, String> {
        let raw_input = target_url.unwrap_or_else(|| "browser://newtab".to_string());
        let resolved_url_str = NavigationResolver::resolve(&raw_input, "google");

        let is_new_tab = resolved_url_str == "browser://newtab" || resolved_url_str == "about:blank";
        let webview_url = if is_new_tab {
            WebviewUrl::App("newtab.html".into())
        } else {
            let parsed = Url::parse(&resolved_url_str)
                .map_err(|e| format!("Invalid URL '{resolved_url_str}': {e}"))?;
            WebviewUrl::External(parsed)
        };

        let tab_id = format!("tab_{}", Uuid::new_v4().simple());
        let webview_label = tab_id.clone();

        // 1. Calculate dynamic positioning
        let layout = BoundsCalculator::compute_layout(window, None)?;

        // 2. Prepare anti-fingerprinting & cosmetic ad defuser script
        let defuser_script = CosmeticFilter::generate_defuser_script();

        // 3. Build child Webview with native hooks & request filtering
        let tab_id_clone = tab_id.clone();
        let app_handle_for_nav = app_handle.clone();
        let app_handle_for_load = app_handle.clone();
        let tab_id_for_load = tab_id.clone();
        let shield_mgr = self.shield_manager.clone();

        let webview_builder = WebviewBuilder::new(&webview_label, webview_url)
            .initialization_script(&defuser_script)
            .on_navigation(move |nav_url| {
                let url_str = nav_url.to_string();
                let mut shields = shield_mgr.lock();

                // Intercept and drop tracking/ad navigation requests
                if shields.is_shield_enabled(&tab_id_clone) {
                    let decision = shields.engine.evaluate_url(&url_str);
                    if decision.should_block {
                        shields.record_blocked_tracker(&tab_id_clone);
                        let stats = shields.get_or_create_stats(&tab_id_clone);
                        let _ = app_handle_for_nav.emit(
                            "shield-stats-updated",
                            ShieldStatsPayload {
                                tab_id: tab_id_clone.clone(),
                                blocked_trackers_count: stats.blocked_trackers_count,
                                blocked_ads_count: stats.blocked_ads_count,
                                https_upgrades_count: stats.https_upgrades_count,
                                shields_enabled: stats.shields_enabled,
                            },
                        );
                        return false; // Cancel navigation
                    }
                }

                let _ = app_handle_for_nav.emit(
                    "tab-navigating",
                    NavigationEventPayload {
                        tab_id: tab_id_clone.clone(),
                        url: url_str,
                    },
                );
                true // Allow navigation
            })
            .on_page_load(move |_webview, payload| {
                let is_loading = matches!(payload.event(), PageLoadEvent::Started);
                let _ = app_handle_for_load.emit(
                    "tab-loading",
                    TabLoadingPayload {
                        tab_id: tab_id_for_load.clone(),
                        url: payload.url().to_string(),
                        is_loading,
                    },
                );
            });

        // 4. Attach WebView directly to parent window client area
        let new_webview = window
            .add_child(
                webview_builder,
                layout.content_rect.position,
                layout.content_rect.size,
            )
            .map_err(|e| format!("Failed to attach child WebView to window: {e}"))?;

        // 5. Hide currently active WebView to avoid frame blending
        if let Some(prev_active_id) = &self.active_tab_id {
            if let Some(prev_wv) = app_handle.get_webview(prev_active_id) {
                let _ = prev_wv.hide();
            }
        }

        // 6. Present the new WebView
        let _ = new_webview.show();

        let mut tab = Tab::new(tab_id.clone(), resolved_url_str);
        if is_new_tab {
            tab.title = "New tab".to_string();
        }

        self.tabs.push(tab.clone());
        self.active_tab_id = Some(tab_id.clone());

        // 7. Initialize shield stats for new tab
        {
            let mut shields = self.shield_manager.lock();
            let stats = shields.get_or_create_stats(&tab_id);
            let _ = app_handle.emit(
                "shield-stats-updated",
                ShieldStatsPayload {
                    tab_id: tab_id.clone(),
                    blocked_trackers_count: stats.blocked_trackers_count,
                    blocked_ads_count: stats.blocked_ads_count,
                    https_upgrades_count: stats.https_upgrades_count,
                    shields_enabled: stats.shields_enabled,
                },
            );
        }

        // 8. Broadcast state changes to the chrome frontend
        let _ = app_handle.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: self.tabs.clone(),
                active_tab_id: self.active_tab_id.clone(),
            },
        );

        Ok(tab)
    }

    /// Switches the active tab by hiding the current WebView and revealing the target WebView.
    pub fn switch_tab(&mut self, app_handle: &AppHandle, target_tab_id: &str) -> Result<(), String> {
        if self.active_tab_id.as_deref() == Some(target_tab_id) {
            return Ok(());
        }

        if !self.tabs.iter().any(|t| t.id == target_tab_id) {
            return Err(format!("Tab ID '{target_tab_id}' does not exist"));
        }

        // 1. Hide current WebView
        if let Some(current_id) = &self.active_tab_id {
            if let Some(current_wv) = app_handle.get_webview(current_id) {
                let _ = current_wv.hide();
            }
        }

        // 2. Reveal and refocus target WebView
        let target_wv = app_handle
            .get_webview(target_tab_id)
            .ok_or_else(|| format!("Native WebView for '{target_tab_id}' not found"))?;

        if let Some(window) = app_handle.get_window("main") {
            if let Ok(layout) = BoundsCalculator::compute_layout(&window, None) {
                let _ = target_wv.set_bounds(layout.content_rect);
            }
        }

        let _ = target_wv.show();
        self.active_tab_id = Some(target_tab_id.to_string());

        // 3. Emit shield stats for target tab
        {
            let mut shields = self.shield_manager.lock();
            let stats = shields.get_or_create_stats(target_tab_id);
            let _ = app_handle.emit(
                "shield-stats-updated",
                ShieldStatsPayload {
                    tab_id: target_tab_id.to_string(),
                    blocked_trackers_count: stats.blocked_trackers_count,
                    blocked_ads_count: stats.blocked_ads_count,
                    https_upgrades_count: stats.https_upgrades_count,
                    shields_enabled: stats.shields_enabled,
                },
            );
        }

        // 4. Emit update event to UI
        let _ = app_handle.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: self.tabs.clone(),
                active_tab_id: self.active_tab_id.clone(),
            },
        );

        Ok(())
    }

    /// Closes a tab and destroys the native OS WebView handle immediately.
    pub fn close_tab(
        &mut self,
        app_handle: &AppHandle,
        window: &Window,
        tab_id: &str,
    ) -> Result<Option<String>, String> {
        let index = self
            .tabs
            .iter()
            .position(|t| t.id == tab_id)
            .ok_or_else(|| format!("Cannot close non-existent tab '{tab_id}'"))?;

        // 1. Destroy native OS WebView handle to release memory immediately
        if let Some(wv) = app_handle.get_webview(tab_id) {
            let _ = wv.close();
        }

        self.tabs.remove(index);

        // 2. Select replacement active tab if closing the currently active one
        let new_active = if self.active_tab_id.as_deref() == Some(tab_id) {
            if self.tabs.is_empty() {
                let replacement = self.create_tab(app_handle, window, None)?;
                Some(replacement.id)
            } else {
                let next_index = if index >= self.tabs.len() {
                    self.tabs.len() - 1
                } else {
                    index
                };
                let next_id = self.tabs[next_index].id.clone();
                self.switch_tab(app_handle, &next_id)?;
                Some(next_id)
            }
        } else {
            self.active_tab_id.clone()
        };

        // 3. Notify chrome frontend
        let _ = app_handle.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: self.tabs.clone(),
                active_tab_id: self.active_tab_id.clone(),
            },
        );

        Ok(new_active)
    }

    pub fn navigate(&mut self, app_handle: &AppHandle, tab_id: &str, input: &str) -> Result<(), String> {
        let resolved_url = NavigationResolver::resolve(input, "google");
        let wv = app_handle.get_webview(tab_id).ok_or("Webview not found")?;

        if resolved_url == "browser://newtab" {
            wv.eval("window.location.href = 'newtab.html';")
                .map_err(|e| format!("Navigation failed: {e}"))?;
        } else {
            let parsed = Url::parse(&resolved_url).map_err(|e| format!("Failed to parse URL: {e}"))?;
            wv.navigate(parsed).map_err(|e| format!("Navigation failed: {e}"))?;
        }

        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.url = resolved_url;
            if tab.url == "browser://newtab" {
                tab.title = "New tab".to_string();
            } else if tab.url.contains("google.com/search") {
                if let Ok(u) = Url::parse(&tab.url) {
                    if let Some((_, q)) = u.query_pairs().find(|(k, _)| k == "q") {
                        tab.title = format!("{q} - Google Search");
                    } else {
                        tab.title = "Google Search".to_string();
                    }
                } else {
                    tab.title = "Google Search".to_string();
                }
            }
        }

        let _ = app_handle.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: self.tabs.clone(),
                active_tab_id: self.active_tab_id.clone(),
            },
        );

        Ok(())
    }

    pub fn reload(&self, app_handle: &AppHandle, tab_id: &str) -> Result<(), String> {
        let wv = app_handle.get_webview(tab_id).ok_or("Webview not found")?;
        wv.eval("window.location.reload();").map_err(|e| e.to_string())
    }

    pub fn go_back(&self, app_handle: &AppHandle, tab_id: &str) -> Result<(), String> {
        let wv = app_handle.get_webview(tab_id).ok_or("Webview not found")?;
        wv.eval("window.history.back();").map_err(|e| e.to_string())
    }

    pub fn go_forward(&self, app_handle: &AppHandle, tab_id: &str) -> Result<(), String> {
        let wv = app_handle.get_webview(tab_id).ok_or("Webview not found")?;
        wv.eval("window.history.forward();").map_err(|e| e.to_string())
    }
}
