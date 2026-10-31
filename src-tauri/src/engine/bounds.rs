use tauri::{LogicalPosition, LogicalSize, Position, Rect, Size, Window};
use crate::tabs::{SidebarMode, SidebarPosition};

/// Default width of the vertical sidebar (240px)
pub const SIDEBAR_WIDTH: f64 = 240.0;
pub const SIDEBAR_COMPACT_WIDTH: f64 = 56.0;
pub const CHROME_BAR_HEIGHT: f64 = 88.0;

#[derive(Debug, Clone, Copy)]
pub struct LayoutBounds {
    pub chrome_rect: Rect,
    pub content_rect: Rect,
    pub topbar_rect: Rect,
}

pub struct BoundsCalculator;

impl BoundsCalculator {
    /// Computes exact pixel-perfect logical bounds for the sidebar (chrome),
    /// the active tab content WebView, and the auto-hiding top bar window controls.
    pub fn compute_layout(
        window: &Window,
        sidebar_width: f64,
        position: SidebarPosition,
        mode: SidebarMode,
        is_overlay_open: bool,
        is_topbar_open: bool,
    ) -> Result<LayoutBounds, String> {
        let physical_size = window
            .inner_size()
            .map_err(|e| format!("Failed to retrieve inner window size: {e}"))?;

        let scale_factor = window
            .scale_factor()
            .map_err(|e| format!("Failed to retrieve scale factor: {e}"))?;

        let logical_size = physical_size.to_logical::<f64>(scale_factor);
        let clamped_width = sidebar_width.max(180.0).min(logical_size.width);

        // Compact floating window controls rectangle (opposite the sidebar)
        let controls_width = 138.0;
        let controls_height = if is_topbar_open { 48.0 } else { 8.0 };

        let topbar_rect = match position {
            SidebarPosition::Left => {
                // Sidebar on Left -> Controls rectangle in top-right corner
                Rect {
                    position: Position::Logical(LogicalPosition::new(
                        (logical_size.width - controls_width).max(0.0),
                        0.0,
                    )),
                    size: Size::Logical(LogicalSize::new(controls_width, controls_height)),
                }
            }
            SidebarPosition::Right => {
                // Sidebar on Right -> Controls rectangle in top-left corner
                Rect {
                    position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                    size: Size::Logical(LogicalSize::new(controls_width, controls_height)),
                }
            }
        };

        match mode {
            SidebarMode::Docked => {
                let content_width = (logical_size.width - clamped_width).max(0.0);
                match position {
                    SidebarPosition::Left => {
                        let chrome_rect = Rect {
                            position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                            size: Size::Logical(LogicalSize::new(clamped_width, logical_size.height)),
                        };
                        let content_rect = Rect {
                            position: Position::Logical(LogicalPosition::new(clamped_width, 0.0)),
                            size: Size::Logical(LogicalSize::new(content_width, logical_size.height)),
                        };
                        Ok(LayoutBounds { chrome_rect, content_rect, topbar_rect })
                    }
                    SidebarPosition::Right => {
                        let content_rect = Rect {
                            position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                            size: Size::Logical(LogicalSize::new(content_width, logical_size.height)),
                        };
                        let chrome_rect = Rect {
                            position: Position::Logical(LogicalPosition::new(content_width, 0.0)),
                            size: Size::Logical(LogicalSize::new(clamped_width, logical_size.height)),
                        };
                        Ok(LayoutBounds { chrome_rect, content_rect, topbar_rect })
                    }
                }
            }
            SidebarMode::AutoHide => {
                let trigger_width = 16.0; // 16px edge trigger strip
                let overlay_width = clamped_width.max(250.0);

                // Content website is ALWAYS 100% full screen in AutoHide mode
                let content_rect = Rect {
                    position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                    size: Size::Logical(LogicalSize::new(logical_size.width, logical_size.height)),
                };

                let current_sidebar_w = if is_overlay_open { overlay_width } else { trigger_width };

                let chrome_rect = match position {
                    SidebarPosition::Left => Rect {
                        position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                        size: Size::Logical(LogicalSize::new(current_sidebar_w, logical_size.height)),
                    },
                    SidebarPosition::Right => Rect {
                        position: Position::Logical(LogicalPosition::new(
                            (logical_size.width - current_sidebar_w).max(0.0),
                            0.0,
                        )),
                        size: Size::Logical(LogicalSize::new(current_sidebar_w, logical_size.height)),
                    },
                };

                Ok(LayoutBounds { chrome_rect, content_rect, topbar_rect })
            }
        }
    }
}
