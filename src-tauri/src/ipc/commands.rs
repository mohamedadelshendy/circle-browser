use tauri::{
    AppHandle, Emitter, Manager, State,
};

use crate::ipc::events::TabStateChangedPayload;
use crate::privacy::{ShieldManagerState, TabShieldStats};
use crate::tabs::{SidebarMode, SidebarPosition, Tab, TabManagerState};

#[tauri::command]
pub fn open_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    url: Option<String>,
) -> Result<Tab, String> {
    let window = app
        .get_window("main")
        .ok_or_else(|| "Parent window 'main' not found".to_string())?;

    state.create_tab(&app, &window, url)
}

#[tauri::command]
pub fn close_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<Option<String>, String> {
    let window = app
        .get_window("main")
        .ok_or_else(|| "Parent window 'main' not found".to_string())?;

    state.close_tab(&app, &window, &tab_id)
}

#[tauri::command]
pub fn switch_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<(), String> {
    state.switch_tab(&app, &tab_id)
}

#[tauri::command]
pub fn reorder_tabs(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    from_index: usize,
    to_index: usize,
) -> Result<(), String> {
    let (all_tabs, active_id) = {
        let mut manager = state.inner.lock();
        if from_index < manager.tabs.len() && to_index < manager.tabs.len() {
            let tab = manager.tabs.remove(from_index);
            manager.tabs.insert(to_index, tab);
        }
        (manager.tabs.clone(), manager.active_tab_id.clone())
    };

    let _ = app.emit(
        "tab-state-changed",
        TabStateChangedPayload {
            tabs: all_tabs,
            active_tab_id: active_id,
        },
    );
    Ok(())
}

#[tauri::command]
pub fn navigate(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
    url: String,
) -> Result<(), String> {
    state.navigate(&app, &tab_id, &url)
}

#[tauri::command]
pub fn reload(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<(), String> {
    state.reload(&app, &tab_id)
}

#[tauri::command]
pub fn go_back(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<(), String> {
    state.go_back(&app, &tab_id)
}

#[tauri::command]
pub fn go_forward(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<(), String> {
    state.go_forward(&app, &tab_id)
}

#[tauri::command]
pub fn get_tabs(state: State<'_, TabManagerState>) -> Result<TabStateChangedPayload, String> {
    let manager = state.inner.lock();
    Ok(TabStateChangedPayload {
        tabs: manager.tabs.clone(),
        active_tab_id: manager.active_tab_id.clone(),
    })
}

// Shield privacy commands
#[tauri::command]
pub fn get_shield_stats(
    state: State<'_, ShieldManagerState>,
    tab_id: String,
) -> Result<TabShieldStats, String> {
    let mut manager = state.0.lock();
    Ok(manager.get_or_create_stats(&tab_id))
}

#[tauri::command]
pub fn toggle_shields(
    app: AppHandle,
    state: State<'_, ShieldManagerState>,
    tab_id: String,
    enabled: bool,
) -> Result<TabShieldStats, String> {
    let mut manager = state.0.lock();
    manager.set_shields_enabled(&tab_id, enabled);
    let stats = manager.get_or_create_stats(&tab_id);
    let _ = app.emit(
        "shield-stats-updated",
        crate::ipc::events::ShieldStatsPayload {
            tab_id: tab_id.clone(),
            blocked_trackers_count: stats.blocked_trackers_count,
            blocked_ads_count: stats.blocked_ads_count,
            https_upgrades_count: stats.https_upgrades_count,
            shields_enabled: stats.shields_enabled,
        },
    );
    Ok(stats)
}

// Sidebar width, mode, overlay and layout control
#[tauri::command]
pub fn set_sidebar_width(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    width: f64,
) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;

    let (pos, mode, is_open) = {
        let mut mgr = state.inner.lock();
        mgr.sidebar_width = width;
        (mgr.sidebar_position, mgr.sidebar_mode, mgr.is_overlay_open)
    };

    let layout = crate::engine::BoundsCalculator::compute_layout(&window, width, pos, mode, is_open)?;
    if let Some(chrome) = app.get_webview("chrome") {
        let _ = chrome.set_bounds(layout.chrome_rect);
    }

    let active_id = {
        let mgr = state.inner.lock();
        mgr.active_tab_id.clone()
    };

    if let Some(tab_id) = active_id {
        if let Some(wv) = app.get_webview(&tab_id) {
            let _ = wv.set_bounds(layout.content_rect);
            let _ = wv.set_focus();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_sidebar_mode(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    mode: String,
) -> Result<String, String> {
    let window = app.get_window("main").ok_or("Main window not found")?;

    let (width, pos, new_mode, is_open) = {
        let mut mgr = state.inner.lock();
        mgr.sidebar_mode = match mode.as_str() {
            "auto_hide" | "autohide" => SidebarMode::AutoHide,
            _ => SidebarMode::Docked,
        };
        mgr.is_overlay_open = false;
        (mgr.sidebar_width, mgr.sidebar_position, mgr.sidebar_mode, mgr.is_overlay_open)
    };

    let layout = crate::engine::BoundsCalculator::compute_layout(&window, width, pos, new_mode, is_open)?;

    if let Some(chrome) = app.get_webview("chrome") {
        let _ = chrome.set_bounds(layout.chrome_rect);
    }

    let active_id = {
        let mgr = state.inner.lock();
        mgr.active_tab_id.clone()
    };

    if let Some(tab_id) = active_id {
        if let Some(wv) = app.get_webview(&tab_id) {
            let _ = wv.set_bounds(layout.content_rect);
            let _ = wv.set_focus();
        }
    }

    let res = match new_mode {
        SidebarMode::Docked => "docked",
        SidebarMode::AutoHide => "auto_hide",
    };
    Ok(res.to_string())
}

#[tauri::command]
pub fn set_sidebar_overlay(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    open: bool,
) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;

    let (width, pos, mode) = {
        let mut mgr = state.inner.lock();
        if mgr.sidebar_mode != SidebarMode::AutoHide {
            return Ok(());
        }
        mgr.is_overlay_open = open;
        (mgr.sidebar_width, mgr.sidebar_position, mgr.sidebar_mode)
    };

    let layout = crate::engine::BoundsCalculator::compute_layout(&window, width, pos, mode, open)?;

    if let Some(chrome) = app.get_webview("chrome") {
        let _ = chrome.set_bounds(layout.chrome_rect);
    }

    // Refocus content webview when overlay closes
    if !open {
        let active_id = {
            let mgr = state.inner.lock();
            mgr.active_tab_id.clone()
        };
        if let Some(tab_id) = active_id {
            if let Some(wv) = app.get_webview(&tab_id) {
                let _ = wv.set_focus();
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn toggle_sidebar_position(
    app: AppHandle,
    state: State<'_, TabManagerState>,
) -> Result<String, String> {
    let window = app.get_window("main").ok_or("Main window not found")?;

    let (width, new_pos, mode, is_open) = {
        let mut mgr = state.inner.lock();
        mgr.sidebar_position = match mgr.sidebar_position {
            SidebarPosition::Left => SidebarPosition::Right,
            SidebarPosition::Right => SidebarPosition::Left,
        };
        (mgr.sidebar_width, mgr.sidebar_position, mgr.sidebar_mode, mgr.is_overlay_open)
    };

    let layout = crate::engine::BoundsCalculator::compute_layout(&window, width, new_pos, mode, is_open)?;
    if let Some(chrome) = app.get_webview("chrome") {
        let _ = chrome.set_bounds(layout.chrome_rect);
    }

    let active_id = {
        let mgr = state.inner.lock();
        mgr.active_tab_id.clone()
    };

    if let Some(tab_id) = active_id {
        if let Some(wv) = app.get_webview(&tab_id) {
            let _ = wv.set_bounds(layout.content_rect);
            let _ = wv.set_focus();
        }
    }

    let pos_str = match new_pos {
        SidebarPosition::Left => "left",
        SidebarPosition::Right => "right",
    };
    Ok(pos_str.to_string())
}

#[tauri::command]
pub fn expand_chrome(_app: AppHandle, _height: f64) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn collapse_chrome(_app: AppHandle) -> Result<(), String> {
    Ok(())
}

// Window control commands for frameless chrome
#[tauri::command]
pub fn minimize_window(app: AppHandle) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn maximize_window(app: AppHandle) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn close_window(app: AppHandle) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    window.close().map_err(|e| e.to_string())
}
