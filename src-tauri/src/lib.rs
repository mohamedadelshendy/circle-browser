pub mod engine;
pub mod ipc;
pub mod privacy;
pub mod storage;
pub mod tabs;

use std::sync::Arc;
use parking_lot::Mutex;
use tauri::{
    webview::WebviewBuilder,
    window::WindowBuilder,
    LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowEvent,
};

use engine::{BoundsCalculator, CHROME_BAR_HEIGHT};
use ipc::commands::*;
use privacy::{ShieldManager, ShieldManagerState};
use tabs::{TabManager, TabManagerState};

pub fn run() {
    let shield_manager = Arc::new(Mutex::new(ShieldManager::new()));
    let shield_manager_state = ShieldManagerState(shield_manager.clone());

    let tab_manager = Arc::new(Mutex::new(TabManager::new(shield_manager.clone())));
    let tab_manager_state = TabManagerState(tab_manager.clone());

    tauri::Builder::default()
        .manage(tab_manager_state)
        .manage(shield_manager_state)
        .invoke_handler(tauri::generate_handler![
            open_tab,
            close_tab,
            switch_tab,
            reorder_tabs,
            navigate,
            reload,
            go_back,
            go_forward,
            get_tabs,
            get_shield_stats,
            toggle_shields,
            expand_chrome,
            collapse_chrome,
            minimize_window,
            maximize_window,
            close_window,
        ])
        .setup(move |app| {
            let app_handle = app.app_handle().clone();

            // 1. Create frameless parent OS window
            let window = WindowBuilder::new(app, "main")
                .title("Circle Browser")
                .inner_size(1280.0, 800.0)
                .min_inner_size(700.0, 500.0)
                .decorations(false) // Frameless to render custom Brave/Chrome-style top bar
                .build()?;

            let win_size = window.inner_size()?;
            let scale = window.scale_factor()?;
            let logical_size = win_size.to_logical::<f64>(scale);

            // 2. Attach UI Chrome Webview at the top
            let chrome_builder = WebviewBuilder::new("chrome", WebviewUrl::App("index.html".into()));
            let _chrome_webview = window.add_child(
                chrome_builder,
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(logical_size.width, CHROME_BAR_HEIGHT),
            )?;

            // 3. Spawn initial default tab
            {
                let mut manager = tab_manager.lock();
                let _ = manager.create_tab(&app_handle, &window, None);
            }

            // 4. Attach window resize listener to synchronize bounds dynamically
            let handle_for_resize = app_handle.clone();
            let tab_mgr_for_resize = tab_manager.clone();

            window.on_window_event(move |event| {
                if let WindowEvent::Resized(_) = event {
                    if let Some(main_win) = handle_for_resize.get_window("main") {
                        if let Ok(layout) = BoundsCalculator::compute_layout(&main_win, None) {
                            // Resize top chrome bar
                            if let Some(chrome) = handle_for_resize.get_webview("chrome") {
                                let _ = chrome.set_bounds(layout.chrome_rect);
                            }

                            // Resize active content WebView
                            let mgr = tab_mgr_for_resize.lock();
                            if let Some(active_id) = &mgr.active_tab_id {
                                if let Some(active_wv) = handle_for_resize.get_webview(active_id) {
                                    let _ = active_wv.set_bounds(layout.content_rect);
                                }
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Titan Browser application");
}
