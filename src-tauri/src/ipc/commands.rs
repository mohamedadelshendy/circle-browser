use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Position, Rect, Size, State,
};

use crate::engine::CHROME_BAR_HEIGHT;
use crate::ipc::events::{ShieldStatsPayload, TabStateChangedPayload};
use crate::privacy::{ShieldManagerState, TabShieldStats};
use crate::tabs::{Tab, TabManagerState};

#[tauri::command]
pub fn open_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    url: Option<String>,
) -> Result<Tab, String> {
    let window = app
        .get_window("main")
        .ok_or_else(|| "Parent window 'main' not found".to_string())?;

    let mut manager = state.0.lock();
    manager.create_tab(&app, &window, url)
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

    let mut manager = state.0.lock();
    manager.close_tab(&app, &window, &tab_id)
}

#[tauri::command]
pub fn switch_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<(), String> {
    let mut manager = state.0.lock();
    manager.switch_tab(&app, &tab_id)
}

#[tauri::command]
pub fn reorder_tabs(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    from_index: usize,
    to_index: usize,
) -> Result<(), String> {
    let mut manager = state.0.lock();
    if from_index < manager.tabs.len() && to_index < manager.tabs.len() {
        let tab = manager.tabs.remove(from_index);
        manager.tabs.insert(to_index, tab);
        let _ = app.emit(
            "tab-state-changed",
            TabStateChangedPayload {
                tabs: manager.tabs.clone(),
                active_tab_id: manager.active_tab_id.clone(),
            },
        );
    }
    Ok(())
}

#[tauri::command]
pub fn navigate(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
    url: String,
) -> Result<(), String> {
    let mut manager = state.0.lock();
    manager.navigate(&app, &tab_id, &url)
}

#[tauri::command]
pub fn reload(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<(), String> {
    let manager = state.0.lock();
    manager.reload(&app, &tab_id)
}

#[tauri::command]
pub fn go_back(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<(), String> {
    let manager = state.0.lock();
    manager.go_back(&app, &tab_id)
}

#[tauri::command]
pub fn go_forward(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    tab_id: String,
) -> Result<(), String> {
    let manager = state.0.lock();
    manager.go_forward(&app, &tab_id)
}

#[tauri::command]
pub fn get_tabs(state: State<'_, TabManagerState>) -> Result<TabStateChangedPayload, String> {
    let manager = state.0.lock();
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
        ShieldStatsPayload {
            tab_id: tab_id.clone(),
            blocked_trackers_count: stats.blocked_trackers_count,
            blocked_ads_count: stats.blocked_ads_count,
            https_upgrades_count: stats.https_upgrades_count,
            shields_enabled: stats.shields_enabled,
        },
    );
    Ok(stats)
}

// Dynamic chrome height control
#[tauri::command]
pub fn expand_chrome(app: AppHandle, height: f64) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let logical = size.to_logical::<f64>(scale);

    if let Some(chrome) = app.get_webview("chrome") {
        let rect = Rect {
            position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
            size: Size::Logical(LogicalSize::new(logical.width, height)),
        };
        chrome.set_bounds(rect).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn collapse_chrome(app: AppHandle) -> Result<(), String> {
    expand_chrome(app, CHROME_BAR_HEIGHT)
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
