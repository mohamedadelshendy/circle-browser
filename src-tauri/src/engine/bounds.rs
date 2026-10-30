use tauri::{LogicalPosition, LogicalSize, Position, Rect, Size, Window};

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
    /// Computes the exact logical bounds for the vertical sidebar (chrome)
    /// and the active tab content webview.
    pub fn compute_layout(
        window: &Window,
        custom_sidebar_width: Option<f64>,
    ) -> Result<LayoutBounds, String> {
        let physical_size = window
            .inner_size()
            .map_err(|e| format!("Failed to retrieve inner window size: {e}"))?;

        let scale_factor = window
            .scale_factor()
            .map_err(|e| format!("Failed to retrieve scale factor: {e}"))?;

        let logical_size = physical_size.to_logical::<f64>(scale_factor);
        let sidebar_width = custom_sidebar_width.unwrap_or(SIDEBAR_WIDTH);

        // Sidebar covers left side, full height (0.0, 0.0) -> (sidebar_width, logical_size.height)
        let chrome_rect = Rect {
            position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
            size: Size::Logical(LogicalSize::new(sidebar_width, logical_size.height)),
        };

        // Floating content canvas sits cleanly on the right with subtle margins
        let margin_x = 4.0;
        let margin_y = 6.0;
        let content_width = (logical_size.width - sidebar_width - margin_x * 2.0).max(0.0);
        let content_height = (logical_size.height - margin_y * 2.0).max(0.0);

        let content_rect = Rect {
            position: Position::Logical(LogicalPosition::new(sidebar_width + margin_x, margin_y)),
            size: Size::Logical(LogicalSize::new(content_width, content_height)),
        };

        Ok(LayoutBounds {
            chrome_rect,
            content_rect,
        })
    }
}
