pub mod bounds;
pub mod navigation;
pub mod zorder;

pub use bounds::{BoundsCalculator, LayoutBounds, CHROME_BAR_HEIGHT, SIDEBAR_COMPACT_WIDTH, SIDEBAR_WIDTH};
pub use navigation::NavigationResolver;
pub use zorder::{bring_chrome_to_top, register_chrome_hwnd, send_tabs_to_bottom};
