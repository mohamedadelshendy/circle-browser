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
    Manager, WebviewUrl, WindowEvent,
};

use engine::{BoundsCalculator, SIDEBAR_WIDTH};
use ipc::commands::*;
use privacy::{ShieldManager, ShieldManagerState};
use tabs::TabManagerState;

pub fn run() {
    let shield_manager = Arc::new(Mutex::new(ShieldManager::new()));
    let shield_manager_state = ShieldManagerState(shield_manager.clone());

    let tab_manager_state = TabManagerState::new(shield_manager.clone());

    tauri::Builder::default()
        .manage(tab_manager_state.clone())
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
            set_sidebar_width,
            toggle_sidebar_position,
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
                .decorations(false) // Frameless to render custom top bar
                .build()?;

            let (init_width, init_pos) = {
                let mgr = tab_manager_state.inner.lock();
                (mgr.sidebar_width, mgr.sidebar_position)
            };
            let layout = BoundsCalculator::compute_layout(&window, init_width, init_pos)?;

            // 2. Attach UI Sidebar Webview
            let chrome_builder = WebviewBuilder::new("chrome", WebviewUrl::App("index.html".into()));
            let _chrome_webview = window.add_child(
                chrome_builder,
                layout.chrome_rect.position,
                layout.chrome_rect.size,
            )?;

            // 3. Spawn initial default tab cleanly without lock contention
            let _ = tab_manager_state.create_tab(&app_handle, &window, None);

            // 4. Attach window resize listener without redundant relayout loops
            let handle_for_resize = app_handle.clone();
            let tab_mgr_for_resize = tab_manager_state.clone();
            let last_physical_size = Arc::new(Mutex::new(tauri::PhysicalSize::new(0, 0)));

            window.on_window_event(move |event| {
                if let WindowEvent::Resized(new_size) = event {
                    {
                        let mut last = last_physical_size.lock();
                        if last.width == new_size.width && last.height == new_size.height {
                            return;
                        }
                        *last = *new_size;
                    }

                    if let Some(main_win) = handle_for_resize.get_window("main") {
                        let (width, pos, active_id) = {
                            if let Some(mgr) = tab_mgr_for_resize.inner.try_lock() {
                                (mgr.sidebar_width, mgr.sidebar_position, mgr.active_tab_id.clone())
                            } else {
                                (SIDEBAR_WIDTH, crate::tabs::SidebarPosition::Left, None)
                            }
                        };

                        if let Ok(layout) = BoundsCalculator::compute_layout(&main_win, width, pos) {
                            if let Some(chrome) = handle_for_resize.get_webview("chrome") {
                                let _ = chrome.set_bounds(layout.chrome_rect);
                            }

                            if let Some(tab_id) = active_id {
                                if let Some(active_wv) = handle_for_resize.get_webview(&tab_id) {
                                    let _ = active_wv.set_bounds(layout.content_rect);
                                    let _ = active_wv.set_focus();
                                }
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Circle Browser application");
}
