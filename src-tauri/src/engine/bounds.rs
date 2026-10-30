use tauri::{LogicalPosition, LogicalSize, Position, Rect, Size, Window};

/// Default height of the custom browser chrome bar (tabs + navigation + omnibox)
pub const CHROME_BAR_HEIGHT: f64 = 76.0;

#[derive(Debug, Clone, Copy)]
pub struct LayoutBounds {
    pub chrome_rect: Rect,
    pub content_rect: Rect,
}

pub struct BoundsCalculator;

impl BoundsCalculator {
    /// Computes the exact logical bounds for the chrome bar and the active tab content
    /// based on the parent window's current dimensions and scale factor.
    ///
    /// The coordinates are calculated relative to the parent window's client viewport:
    /// - Chrome bar: fixed at (x: 0, y: 0) with height `CHROME_BAR_HEIGHT` and full window width.
    /// - Content WebView: positioned at (x: 0, y: CHROME_BAR_HEIGHT) spanning the remaining height.
    pub fn compute_layout(
        window: &Window,
        custom_chrome_height: Option<f64>,
    ) -> Result<LayoutBounds, String> {
        let physical_size = window
            .inner_size()
            .map_err(|e| format!("Failed to retrieve inner window size: {e}"))?;

        let scale_factor = window
            .scale_factor()
            .map_err(|e| format!("Failed to retrieve scale factor: {e}"))?;

        let logical_size = physical_size.to_logical::<f64>(scale_factor);
        let chrome_height = custom_chrome_height.unwrap_or(CHROME_BAR_HEIGHT);

        let chrome_rect = Rect {
            position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
            size: Size::Logical(LogicalSize::new(logical_size.width, chrome_height)),
        };

        let content_height = (logical_size.height - chrome_height).max(0.0);
        let content_rect = Rect {
            position: Position::Logical(LogicalPosition::new(0.0, chrome_height)),
            size: Size::Logical(LogicalSize::new(logical_size.width, content_height)),
        };

        Ok(LayoutBounds {
            chrome_rect,
            content_rect,
        })
    }
}

