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

use engine::BoundsCalculator;
use ipc::commands::*;
use privacy::{ShieldManager, ShieldManagerState};
use tabs::TabManagerState;

pub fn run() {
    let shield_manager = Arc::new(Mutex::new(ShieldManager::new()));
    let shield_manager_state = ShieldManagerState(shield_manager.clone());

    let settings_path = storage::get_settings_path(None);
    let saved_settings = storage::load_settings(&settings_path);
    let tab_manager_state = TabManagerState::new(shield_manager.clone(), saved_settings);

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
            set_sidebar_mode,
            set_sidebar_overlay,
            set_topbar_overlay,
            toggle_sidebar_position,
            set_sidebar_position,
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
                .theme(Some(tauri::Theme::Dark))
                .decorations(false) // Frameless to render custom top bar
                .transparent(true)
                .build()?;

            let (init_width, init_pos, init_mode, is_open, is_topbar_open) = {
                let mgr = tab_manager_state.inner.lock();
                (mgr.sidebar_width, mgr.sidebar_position, mgr.sidebar_mode, mgr.is_overlay_open, mgr.is_topbar_open)
            };
            let layout = BoundsCalculator::compute_layout(&window, init_width, init_pos, init_mode, is_open, is_topbar_open)?;

            // 2. Attach UI Sidebar Webview with transparency so only the floating card renders
            let chrome_builder = WebviewBuilder::new("chrome", WebviewUrl::App("index.html".into()))
                .transparent(true);
            let _chrome_webview = window.add_child(
                chrome_builder,
                layout.chrome_rect.position,
                layout.chrome_rect.size,
            )?;

            // 3. Attach TopBar Window Controls Webview with transparency
            let topbar_builder = WebviewBuilder::new("topbar", WebviewUrl::App("index.html?view=topbar".into()))
                .transparent(true);
            let _topbar_webview = window.add_child(
                topbar_builder,
                layout.topbar_rect.position,
                layout.topbar_rect.size,
            )?;

            engine::register_ui_hwnds(&window);

            // 4. Spawn initial default tab cleanly without lock contention
            let _ = tab_manager_state.create_tab(&app_handle, &window, None);
            engine::send_tabs_to_bottom(&window);
            engine::bring_topbar_to_top();
            engine::bring_chrome_to_top();

            // 5. Attach window resize listener without redundant relayout loops
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
                        let (width, pos, mode, is_open, is_topbar_open, active_id) = {
                            let mgr = tab_mgr_for_resize.inner.lock();
                            (mgr.sidebar_width, mgr.sidebar_position, mgr.sidebar_mode, mgr.is_overlay_open, mgr.is_topbar_open, mgr.active_tab_id.clone())
                        };

                        if let Ok(layout) = BoundsCalculator::compute_layout(&main_win, width, pos, mode, is_open, is_topbar_open) {
                            if let Some(chrome) = handle_for_resize.get_webview("chrome") {
                                let _ = chrome.set_bounds(layout.chrome_rect);
                            }

                            if let Some(topbar) = handle_for_resize.get_webview("topbar") {
                                let _ = topbar.set_bounds(layout.topbar_rect);
                            }

                            if let Some(tab_id) = active_id {
                                if let Some(active_wv) = handle_for_resize.get_webview(&tab_id) {
                                    let _ = active_wv.set_bounds(layout.content_rect);
                                }
                            }
                            engine::send_tabs_to_bottom(&main_win);
                            engine::bring_topbar_to_top();
                            engine::bring_chrome_to_top();
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Circle Browser application");
}
