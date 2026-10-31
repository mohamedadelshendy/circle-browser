use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{
    webview::{PageLoadEvent, WebviewBuilder},
    AppHandle, Emitter, Manager, Url, WebviewUrl, Window,
};
use uuid::Uuid;

use crate::engine::{BoundsCalculator, NavigationResolver, SIDEBAR_WIDTH};
use crate::ipc::events::{
    NavigationEventPayload, ShieldStatsPayload, TabLoadingPayload, TabStateChangedPayload,
};
use crate::privacy::{CosmeticFilter, ShieldManager};
use crate::tabs::tab::Tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarMode {
    Docked,
    AutoHide,
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<String>,
    pub sidebar_width: f64,
    pub sidebar_position: SidebarPosition,
    pub sidebar_mode: SidebarMode,
    pub is_overlay_open: bool,
    pub is_topbar_open: bool,
}

#[derive(Clone)]
pub struct TabManagerState {
    pub inner: Arc<Mutex<TabManager>>,
    pub shield_manager: Arc<Mutex<ShieldManager>>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            sidebar_width: SIDEBAR_WIDTH,
            sidebar_position: SidebarPosition::Left,
            sidebar_mode: SidebarMode::Docked,
            is_overlay_open: false,
            is_topbar_open: false,
        }
    }

    pub fn with_settings(settings: crate::storage::AppSettings) -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            sidebar_width: settings.sidebar_width,
            sidebar_position: settings.sidebar_position,
            sidebar_mode: settings.sidebar_mode,
            is_overlay_open: false,
            is_topbar_open: false,
        }
    }
}

impl TabManagerState {
    pub fn new(shield_manager: Arc<Mutex<ShieldManager>>, settings: crate::storage::AppSettings) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TabManager::with_settings(settings))),
            shield_manager,
        }
    }

    /// Spawns a child WebView cleanly WITHOUT holding any Mutex locks during native OS calls.
    pub fn create_tab(
        &self,
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

        let (current_width, current_pos, current_mode, is_open, is_topbar_open) = {
            let mgr = self.inner.lock();
            (mgr.sidebar_width, mgr.sidebar_position, mgr.sidebar_mode, mgr.is_overlay_open, mgr.is_topbar_open)
        };

        // 1. Calculate dynamic positioning with zero gap
        let layout = BoundsCalculator::compute_layout(window, current_width, current_pos, current_mode, is_open, is_topbar_open)?;

        // 2. Prepare anti-fingerprinting & cosmetic ad defuser script
        let defuser_script = CosmeticFilter::generate_defuser_script();

        // 3. Register new tab in manager state before attaching webview
        let mut new_tab = Tab::new(tab_id.clone(), resolved_url_str.clone());
        if is_new_tab {
            new_tab.title = "New tab".to_string();
            new_tab.favicon = None;
        } else if let Ok(parsed) = Url::parse(&resolved_url_str) {
            if let Some(host) = parsed.host_str() {
                new_tab.favicon = Some(format!("https://www.google.com/s2/favicons?domain={host}&sz=32"));
            }
        }

        let prev_active_id = {
            let mut manager = self.inner.lock();
            let prev = manager.active_tab_id.clone();
            manager.tabs.push(new_tab.clone());
            manager.active_tab_id = Some(tab_id.clone());
            prev
        };

        // 4. Build child Webview with native hooks & request filtering
        let tab_id_clone = tab_id.clone();
        let app_handle_for_nav = app_handle.clone();
        let app_handle_for_load = app_handle.clone();
        let tab_id_for_load = tab_id.clone();
        let shield_mgr = self.shield_manager.clone();
        let inner_for_nav = self.inner.clone();
        let inner_for_load = self.inner.clone();

        let webview_builder = WebviewBuilder::new(&webview_label, webview_url)
            .initialization_script(&defuser_script)
            .on_navigation(move |nav_url| {
                let url_str = nav_url.to_string();
                let mut shields = shield_mgr.lock();

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
                        return false;
                    }
                }

                // Update tab state in manager atomically
                {
                    let mut mgr = inner_for_nav.lock();
                    if let Some(tab) = mgr.tabs.iter_mut().find(|t| t.id == tab_id_clone) {
                        tab.url = url_str.clone();
                        tab.title = NavigationResolver::title_from_url(&url_str);
                        if let Ok(parsed) = Url::parse(&url_str) {
                            if let Some(host) = parsed.host_str() {
                                tab.favicon = Some(format!("https://www.google.com/s2/favicons?domain={host}&sz=32"));
                            }
                        }
                    }
                }

                let _ = app_handle_for_nav.emit(
                    "tab-navigating",
                    NavigationEventPayload {
                        tab_id: tab_id_clone.clone(),
                        url: url_str,
                    },
                );

                true
            })
            .on_page_load(move |_webview, payload| {
                let is_loading = matches!(payload.event(), PageLoadEvent::Started);
                let url_str = payload.url().to_string();

                {
                    let mut mgr = inner_for_load.lock();
                    if let Some(tab) = mgr.tabs.iter_mut().find(|t| t.id == tab_id_for_load) {
                        tab.url = url_str.clone();
                        tab.is_loading = is_loading;
                        if !is_loading {
                            tab.title = NavigationResolver::title_from_url(&url_str);
                            if let Ok(parsed) = Url::parse(&url_str) {
                                if let Some(host) = parsed.host_str() {
                                    tab.favicon = Some(format!("https://www.google.com/s2/favicons?domain={host}&sz=32"));
                                }
                            }
                        }
                    }
                }

                let _ = app_handle_for_load.emit(
                    "tab-loading",
                    TabLoadingPayload {
                        tab_id: tab_id_for_load.clone(),
                        url: url_str,
                        is_loading,
                    },
                );
            });

        // 5. Attach WebView directly to window
        let new_webview = match window.add_child(
            webview_builder,
            layout.content_rect.position,
            layout.content_rect.size,
        ) {
            Ok(wv) => wv,
            Err(e) => {
                // Rollback tab from manager if window.add_child failed
                let mut manager = self.inner.lock();
                manager.tabs.retain(|t| t.id != tab_id);
                manager.active_tab_id = prev_active_id;
                return Err(format!("Failed to attach child WebView to window: {e}"));
            }
        };

        // 6. Hide previous and show new WebView + ensure it has focus
        if let Some(prev_id) = prev_active_id {
            if let Some(prev_wv) = app_handle.get_webview(&prev_id) {
                let _ = prev_wv.hide();
            }
        }
        let _ = new_webview.show();
        let _ = new_webview.set_focus();
        crate::engine::send_tabs_to_bottom(window);
        crate::engine::bring_topbar_to_top();
        crate::engine::bring_chrome_to_top();

        // 7. Initialize shield stats
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

        // 8. Broadcast state changes
        let (all_tabs, active_id) = {
            let manager = self.inner.lock();
            (manager.tabs.clone(), manager.active_tab_id.clone())
        };

        let _ = app_handle.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: all_tabs,
                active_tab_id: active_id,
            },
        );

        Ok(new_tab)
    }

    /// Switches tab without holding locks during OS show/hide operations.
    pub fn switch_tab(&self, app_handle: &AppHandle, target_tab_id: &str) -> Result<(), String> {
        let (current_id, target_exists) = {
            let mgr = self.inner.lock();
            if mgr.active_tab_id.as_deref() == Some(target_tab_id) {
                return Ok(());
            }
            let exists = mgr.tabs.iter().any(|t| t.id == target_tab_id);
            (mgr.active_tab_id.clone(), exists)
        };

        if !target_exists {
            return Err(format!("Tab ID '{target_tab_id}' does not exist"));
        }

        // 1. Hide current WebView
        if let Some(current_id) = current_id {
            if let Some(current_wv) = app_handle.get_webview(&current_id) {
                let _ = current_wv.hide();
            }
        }

        // 2. Reveal and refocus target WebView
        let target_wv = app_handle
            .get_webview(target_tab_id)
            .ok_or_else(|| format!("Native WebView for '{target_tab_id}' not found"))?;

        let (width, pos, mode, is_open, is_topbar_open) = {
            let mgr = self.inner.lock();
            (mgr.sidebar_width, mgr.sidebar_position, mgr.sidebar_mode, mgr.is_overlay_open, mgr.is_topbar_open)
        };

        if let Some(window) = app_handle.get_window("main") {
            if let Ok(layout) = BoundsCalculator::compute_layout(&window, width, pos, mode, is_open, is_topbar_open) {
                let _ = target_wv.set_bounds(layout.content_rect);
            }
        }

        let _ = target_wv.show();
        let _ = target_wv.set_focus();
        if let Some(window) = app_handle.get_window("main") {
            crate::engine::send_tabs_to_bottom(&window);
            crate::engine::bring_topbar_to_top();
            crate::engine::bring_chrome_to_top();
        }

        // 3. Update active tab id
        let (all_tabs, active_id) = {
            let mut mgr = self.inner.lock();
            mgr.active_tab_id = Some(target_tab_id.to_string());
            (mgr.tabs.clone(), mgr.active_tab_id.clone())
        };

        // 4. Emit shield stats for target tab
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

        // 5. Emit update event to UI
        let _ = app_handle.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: all_tabs,
                active_tab_id: active_id,
            },
        );

        Ok(())
    }

    /// Closes a tab without holding locks during OS destroy calls.
    pub fn close_tab(&self, app_handle: &AppHandle, window: &Window, tab_id: &str) -> Result<Option<String>, String> {
        let (next_active_id, should_spawn_new) = {
            let mut mgr = self.inner.lock();
            let index = mgr
                .tabs
                .iter()
                .position(|t| t.id == tab_id)
                .ok_or_else(|| format!("Cannot close non-existent tab '{tab_id}'"))?;

            mgr.tabs.remove(index);

            if mgr.active_tab_id.as_deref() == Some(tab_id) {
                if mgr.tabs.is_empty() {
                    (None, true)
                } else {
                    let next_index = if index >= mgr.tabs.len() {
                        mgr.tabs.len() - 1
                    } else {
                        index
                    };
                    let next_id = mgr.tabs[next_index].id.clone();
                    (Some(next_id), false)
                }
            } else {
                (mgr.active_tab_id.clone(), false)
            }
        };

        // 1. Destroy native OS WebView handle without holding lock
        if let Some(wv) = app_handle.get_webview(tab_id) {
            let _ = wv.close();
        }

        // 2. Select replacement active tab
        let final_active = if should_spawn_new {
            let replacement = self.create_tab(app_handle, window, None)?;
            Some(replacement.id)
        } else if let Some(next_id) = next_active_id {
            self.switch_tab(app_handle, &next_id)?;
            Some(next_id)
        } else {
            None
        };

        let (all_tabs, active_id) = {
            let mgr = self.inner.lock();
            (mgr.tabs.clone(), mgr.active_tab_id.clone())
        };

        // 3. Notify chrome frontend
        let _ = app_handle.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: all_tabs,
                active_tab_id: active_id,
            },
        );

        Ok(final_active)
    }

    pub fn navigate(&self, app_handle: &AppHandle, tab_id: &str, input: &str) -> Result<(), String> {
        let resolved_url = NavigationResolver::resolve(input, "google");
        let wv = app_handle.get_webview(tab_id).ok_or("Webview not found")?;

        if resolved_url == "browser://newtab" {
            wv.eval("window.location.href = 'newtab.html';")
                .map_err(|e| format!("Navigation failed: {e}"))?;
        } else {
            let parsed = Url::parse(&resolved_url).map_err(|e| format!("Failed to parse URL: {e}"))?;
            wv.navigate(parsed).map_err(|e| format!("Navigation failed: {e}"))?;
        }
        let _ = wv.set_focus();

        let (all_tabs, active_id) = {
            let mut mgr = self.inner.lock();
            if let Some(tab) = mgr.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.url = resolved_url.clone();
                tab.title = NavigationResolver::title_from_url(&resolved_url);
                if let Ok(parsed) = Url::parse(&resolved_url) {
                    if let Some(host) = parsed.host_str() {
                        tab.favicon = Some(format!("https://www.google.com/s2/favicons?domain={host}&sz=32"));
                    }
                }
            }
            (mgr.tabs.clone(), mgr.active_tab_id.clone())
        };

        let _ = app_handle.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: all_tabs,
                active_tab_id: active_id,
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
        wv.eval("try { window.history.back(); } catch (e) { console.error('Back navigation failed:', e); }")
            .map_err(|e| e.to_string())
    }

    pub fn go_forward(&self, app_handle: &AppHandle, tab_id: &str) -> Result<(), String> {
        let wv = app_handle.get_webview(tab_id).ok_or("Webview not found")?;
        wv.eval("try { window.history.forward(); } catch (e) { console.error('Forward navigation failed:', e); }")
            .map_err(|e| e.to_string())
    }
}
