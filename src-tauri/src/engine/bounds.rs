use tauri::{LogicalPosition, LogicalSize, Position, Rect, Size, Window};
use crate::tabs::SidebarPosition;

/// Default width of the vertical sidebar (240px)
pub const SIDEBAR_WIDTH: f64 = 240.0;
pub const SIDEBAR_COMPACT_WIDTH: f64 = 56.0;
pub const CHROME_BAR_HEIGHT: f64 = 88.0;

#[derive(Debug, Clone, Copy)]
pub struct LayoutBounds {
    pub chrome_rect: Rect,
    pub content_rect: Rect,
}

pub struct BoundsCalculator;

impl BoundsCalculator {
    /// Computes exact pixel-perfect logical bounds for the sidebar (chrome)
    /// and the active tab content WebView with zero gaps to prevent mouse drop issues.
    pub fn compute_layout(
        window: &Window,
        sidebar_width: f64,
        position: SidebarPosition,
    ) -> Result<LayoutBounds, String> {
        let physical_size = window
            .inner_size()
            .map_err(|e| format!("Failed to retrieve inner window size: {e}"))?;

        let scale_factor = window
            .scale_factor()
            .map_err(|e| format!("Failed to retrieve scale factor: {e}"))?;

        let logical_size = physical_size.to_logical::<f64>(scale_factor);
        let clamped_width = sidebar_width.max(0.0).min(logical_size.width);
        let content_width = (logical_size.width - clamped_width).max(0.0);

        match position {
            SidebarPosition::Left => {
                // Sidebar pinned to the left edge, full height
                let chrome_rect = Rect {
                    position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                    size: Size::Logical(LogicalSize::new(clamped_width, logical_size.height)),
                };

                // Content WebView fills the remaining right side completely (zero dead-zone gaps)
                let content_rect = Rect {
                    position: Position::Logical(LogicalPosition::new(clamped_width, 0.0)),
                    size: Size::Logical(LogicalSize::new(content_width, logical_size.height)),
                };

                Ok(LayoutBounds {
                    chrome_rect,
                    content_rect,
                })
            }
            SidebarPosition::Right => {
                // Content WebView fills the left side
                let content_rect = Rect {
                    position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                    size: Size::Logical(LogicalSize::new(content_width, logical_size.height)),
                };

                // Sidebar pinned to the right edge
                let chrome_rect = Rect {
                    position: Position::Logical(LogicalPosition::new(content_width, 0.0)),
                    size: Size::Logical(LogicalSize::new(clamped_width, logical_size.height)),
                };

                Ok(LayoutBounds {
                    chrome_rect,
                    content_rect,
                })
            }
        }
    }
}
