//! Win32 `ClipCursor` mouse capture, cursor centering, and window maximization.

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ClipCursor, GetClientRect, GetCursorPos, SetCursorPos, ShowWindow, SW_MAXIMIZE,
};

pub struct Win32Capture {
    hwnd: HWND,
    grabbed: bool,
    center_x: i32,
    center_y: i32,
}

impl Win32Capture {
    pub fn new(window: &minifb::Window, width: i32, height: i32) -> Option<Win32Capture> {
        let window_handle = window.window_handle().ok()?;
        let hwnd = match window_handle.as_raw() {
            RawWindowHandle::Win32(h) => h.hwnd.get() as HWND,
            _ => return None,
        };

        Some(Win32Capture {
            hwnd,
            grabbed: false,
            center_x: width / 2,
            center_y: height / 2,
        })
    }

    pub fn set_captured(&mut self, captured: bool) {
        if captured == self.grabbed {
            return;
        }
        unsafe {
            if captured {
                let mut rect: RECT = std::mem::zeroed();
                if GetClientRect(self.hwnd, &mut rect) != 0 {
                    let mut pt_top_left = POINT { x: rect.left, y: rect.top };
                    let mut pt_bottom_right = POINT { x: rect.right, y: rect.bottom };
                    ClientToScreen(self.hwnd, &mut pt_top_left);
                    ClientToScreen(self.hwnd, &mut pt_bottom_right);
                    let screen_rect = RECT {
                        left: pt_top_left.x,
                        top: pt_top_left.y,
                        right: pt_bottom_right.x,
                        bottom: pt_bottom_right.y,
                    };
                    ClipCursor(&screen_rect);
                }
                self.warp_to_center();
            } else {
                ClipCursor(std::ptr::null());
            }
        }
        self.grabbed = captured;
    }

    #[allow(dead_code)]
    pub fn is_captured(&self) -> bool {
        self.grabbed
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.center_x = width / 2;
        self.center_y = height / 2;
    }

    fn warp_to_center(&self) {
        unsafe {
            let mut pt = POINT { x: self.center_x, y: self.center_y };
            ClientToScreen(self.hwnd, &mut pt);
            SetCursorPos(pt.x, pt.y);
        }
    }

    pub fn poll_delta_and_recenter(&self) -> (f32, f32) {
        if !self.grabbed {
            return (0.0, 0.0);
        }
        unsafe {
            let mut cur_pos: POINT = std::mem::zeroed();
            if GetCursorPos(&mut cur_pos) == 0 {
                return (0.0, 0.0);
            }
            let mut center_pt = POINT { x: self.center_x, y: self.center_y };
            ClientToScreen(self.hwnd, &mut center_pt);

            let dx = (cur_pos.x - center_pt.x) as f32;
            let dy = (cur_pos.y - center_pt.y) as f32;

            if cur_pos.x != center_pt.x || cur_pos.y != center_pt.y {
                self.warp_to_center();
            }

            (dx, dy)
        }
    }
}

impl Drop for Win32Capture {
    fn drop(&mut self) {
        if self.grabbed {
            unsafe {
                ClipCursor(std::ptr::null());
            }
        }
    }
}

pub fn maximize_win32(window: &minifb::Window) {
    if let Ok(window_handle) = window.window_handle() {
        if let RawWindowHandle::Win32(h) = window_handle.as_raw() {
            let hwnd = h.hwnd.get() as HWND;
            unsafe {
                ShowWindow(hwnd, SW_MAXIMIZE);
            }
        }
    }
}

