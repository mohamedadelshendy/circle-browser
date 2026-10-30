use std::sync::atomic::{AtomicIsize, Ordering};

pub static CHROME_HWND: AtomicIsize = AtomicIsize::new(0);
pub static TOPBAR_HWND: AtomicIsize = AtomicIsize::new(0);

#[cfg(target_os = "windows")]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
    pub type UINT = u32;

    pub const HWND_TOP: HWND = 0 as HWND;
    pub const HWND_BOTTOM: HWND = 1 as HWND;
    pub const SWP_NOSIZE: UINT = 0x0001;
    pub const SWP_NOMOVE: UINT = 0x0002;
    pub const SWP_NOACTIVATE: UINT = 0x0010;

    extern "system" {
        pub fn SetWindowPos(
            hWnd: HWND,
            hWndInsertAfter: HWND,
            X: i32,
            Y: i32,
            cx: i32,
            cy: i32,
            uFlags: UINT,
        ) -> BOOL;
        pub fn FindWindowExW(
            hWndParent: HWND,
            hWndChildAfter: HWND,
            lpszClass: *const u16,
            lpszWindow: *const u16,
        ) -> HWND;
    }
}

/// Registers the chrome and topbar webview HWNDs right after they are created.
pub fn register_ui_hwnds(window: &tauri::Window) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(parent) = window.hwnd() {
            let parent_ptr = parent.0 as *mut std::ffi::c_void;
            unsafe {
                let first_child = win32::FindWindowExW(
                    parent_ptr,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    std::ptr::null(),
                );
                if !first_child.is_null() {
                    CHROME_HWND.store(first_child as isize, Ordering::SeqCst);

                    let second_child = win32::FindWindowExW(
                        parent_ptr,
                        first_child,
                        std::ptr::null(),
                        std::ptr::null(),
                    );
                    if !second_child.is_null() {
                        TOPBAR_HWND.store(second_child as isize, Ordering::SeqCst);
                    }
                }
            }
        }
    }
}

/// Backward compatibility alias
pub fn register_chrome_hwnd(window: &tauri::Window) {
    register_ui_hwnds(window);
}

/// Brings topbar to HWND_TOP so it renders on top of web pages.
pub fn bring_topbar_to_top() {
    #[cfg(target_os = "windows")]
    {
        let hwnd = TOPBAR_HWND.load(Ordering::SeqCst);
        if hwnd != 0 {
            unsafe {
                win32::SetWindowPos(
                    hwnd as *mut std::ffi::c_void,
                    win32::HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    win32::SWP_NOMOVE | win32::SWP_NOSIZE | win32::SWP_NOACTIVATE,
                );
            }
        }
    }
}

/// Brings chrome (sidebar UI) to HWND_TOP above everything (including topbar).
pub fn bring_chrome_to_top() {
    #[cfg(target_os = "windows")]
    {
        let hwnd = CHROME_HWND.load(Ordering::SeqCst);
        if hwnd != 0 {
            unsafe {
                win32::SetWindowPos(
                    hwnd as *mut std::ffi::c_void,
                    win32::HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    win32::SWP_NOMOVE | win32::SWP_NOSIZE | win32::SWP_NOACTIVATE,
                );
            }
        }
    }
}

/// Pushes all tab webviews to HWND_BOTTOM so they are always the layer below chrome and topbar.
pub fn send_tabs_to_bottom(window: &tauri::Window) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(parent) = window.hwnd() {
            let parent_ptr = parent.0 as *mut std::ffi::c_void;
            let chrome_ptr = CHROME_HWND.load(Ordering::SeqCst) as *mut std::ffi::c_void;
            let topbar_ptr = TOPBAR_HWND.load(Ordering::SeqCst) as *mut std::ffi::c_void;
            unsafe {
                let mut child = win32::FindWindowExW(
                    parent_ptr,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    std::ptr::null(),
                );
                while !child.is_null() {
                    if child != chrome_ptr && child != topbar_ptr {
                        win32::SetWindowPos(
                            child,
                            win32::HWND_BOTTOM,
                            0,
                            0,
                            0,
                            0,
                            win32::SWP_NOMOVE | win32::SWP_NOSIZE | win32::SWP_NOACTIVATE,
                        );
                    }
                    child = win32::FindWindowExW(
                        parent_ptr,
                        child,
                        std::ptr::null(),
                        std::ptr::null(),
                    );
                }
            }
        }
    }
}

