// Cross-platform mouse capture and window management.
//
// Supports:
// - Linux / X11 (via x11-dl: XGrabPointer, XWarpPointer, transparent cursor)
// - Windows (via windows-sys: ClipCursor, SetCursorPos, ShowCursor)
// - Fallback for Wayland / macOS / non-native environments (delta tracking)

use minifb::Window;

#[cfg(target_os = "linux")]
mod x11_capture {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
    use std::os::raw::{c_int, c_ulong};
    use x11_dl::xlib;

    pub struct X11Capture {
        lib: xlib::Xlib,
        display: *mut xlib::Display,
        window: c_ulong,
        blank_cursor: c_ulong,
        grabbed: bool,
        center_x: i32,
        center_y: i32,
    }

    impl X11Capture {
        pub fn new(window: &minifb::Window, width: i32, height: i32) -> Option<X11Capture> {
            let display_handle = window.display_handle().ok()?;
            let window_handle = window.window_handle().ok()?;

            let xlib_display = match display_handle.as_raw() {
                RawDisplayHandle::Xlib(h) => h,
                _ => return None,
            };
            let xlib_window = match window_handle.as_raw() {
                RawWindowHandle::Xlib(h) => h,
                _ => return None,
            };

            let lib = xlib::Xlib::open().ok()?;
            let display = xlib_display.display?.as_ptr() as *mut xlib::Display;
            let xid = xlib_window.window as c_ulong;

            let blank_cursor = unsafe { create_blank_cursor(&lib, display, xid) };

            Some(X11Capture {
                lib,
                display,
                window: xid,
                blank_cursor,
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
                    (self.lib.XDefineCursor)(self.display, self.window, self.blank_cursor);
                    (self.lib.XGrabPointer)(
                        self.display,
                        self.window,
                        xlib::True,
                        (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask)
                            as u32,
                        xlib::GrabModeAsync,
                        xlib::GrabModeAsync,
                        self.window,
                        self.blank_cursor,
                        xlib::CurrentTime,
                    );
                    self.warp_to_center();
                } else {
                    (self.lib.XUngrabPointer)(self.display, xlib::CurrentTime);
                    (self.lib.XUndefineCursor)(self.display, self.window);
                }
                (self.lib.XFlush)(self.display);
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
                (self.lib.XWarpPointer)(
                    self.display,
                    0,
                    self.window,
                    0,
                    0,
                    0,
                    0,
                    self.center_x,
                    self.center_y,
                );
                (self.lib.XFlush)(self.display);
            }
        }

        pub fn poll_delta_and_recenter(&self) -> (f32, f32) {
            if !self.grabbed {
                return (0.0, 0.0);
            }
            unsafe {
                let mut root_ret: c_ulong = 0;
                let mut child_ret: c_ulong = 0;
                let mut root_x: c_int = 0;
                let mut root_y: c_int = 0;
                let mut win_x: c_int = 0;
                let mut win_y: c_int = 0;
                let mut mask: u32 = 0;

                let ok = (self.lib.XQueryPointer)(
                    self.display,
                    self.window,
                    &mut root_ret,
                    &mut child_ret,
                    &mut root_x,
                    &mut root_y,
                    &mut win_x,
                    &mut win_y,
                    &mut mask,
                );

                if ok == 0 {
                    return (0.0, 0.0);
                }

                let dx = (win_x - self.center_x) as f32;
                let dy = (win_y - self.center_y) as f32;

                if win_x != self.center_x || win_y != self.center_y {
                    self.warp_to_center();
                }

                (dx, dy)
            }
        }
    }

    impl Drop for X11Capture {
        fn drop(&mut self) {
            unsafe {
                if self.grabbed {
                    (self.lib.XUngrabPointer)(self.display, xlib::CurrentTime);
                }
                (self.lib.XFreeCursor)(self.display, self.blank_cursor);
                (self.lib.XFlush)(self.display);
            }
        }
    }

    unsafe fn create_blank_cursor(
        lib: &xlib::Xlib,
        display: *mut xlib::Display,
        window: c_ulong,
    ) -> c_ulong {
        let data: [u8; 1] = [0];
        let pixmap = (lib.XCreateBitmapFromData)(display, window, data.as_ptr() as *const i8, 1, 1);
        let mut color: xlib::XColor = std::mem::zeroed();
        let cursor = (lib.XCreatePixmapCursor)(display, pixmap, pixmap, &mut color, &mut color, 0, 0);
        (lib.XFreePixmap)(display, pixmap);
        cursor
    }

    pub fn maximize_x11(window: &minifb::Window) {
        let (Ok(display_handle), Ok(window_handle)) = (window.display_handle(), window.window_handle()) else {
            return;
        };

        let xlib_display = match display_handle.as_raw() {
            RawDisplayHandle::Xlib(h) => h,
            _ => return,
        };
        let xlib_window = match window_handle.as_raw() {
            RawWindowHandle::Xlib(h) => h,
            _ => return,
        };

        let Ok(lib) = xlib::Xlib::open() else {
            return;
        };
        let Some(display_ptr) = xlib_display.display else {
            return;
        };
        let display = display_ptr.as_ptr() as *mut xlib::Display;
        let xid = xlib_window.window as c_ulong;

        unsafe {
            let wm_state = (lib.XInternAtom)(display, b"_NET_WM_STATE\0".as_ptr() as *const _, xlib::False);
            let max_vert = (lib.XInternAtom)(display, b"_NET_WM_STATE_MAXIMIZED_VERT\0".as_ptr() as *const _, xlib::False);
            let max_horz = (lib.XInternAtom)(display, b"_NET_WM_STATE_MAXIMIZED_HORZ\0".as_ptr() as *const _, xlib::False);

            let atoms = [max_vert, max_horz];
            (lib.XChangeProperty)(
                display,
                xid,
                wm_state,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                atoms.as_ptr() as *const _,
                2,
            );

            let root = (lib.XDefaultRootWindow)(display);
            let mut xev: xlib::XClientMessageEvent = std::mem::zeroed();
            xev.type_ = xlib::ClientMessage;
            xev.window = xid;
            xev.message_type = wm_state;
            xev.format = 32;
            let longs = xev.data.as_longs_mut();
            longs[0] = 1;
            longs[1] = max_vert as std::os::raw::c_long;
            longs[2] = max_horz as std::os::raw::c_long;
            longs[3] = 1;
            longs[4] = 0;

            let mut event: xlib::XEvent = std::mem::zeroed();
            event.client_message = xev;
            (lib.XSendEvent)(
                display,
                root,
                xlib::False,
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
                &mut event,
            );
            (lib.XFlush)(display);
        }
    }
}

#[cfg(windows)]
mod win32_capture {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::Foundation::{HWND, RECT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        ClipCursor, GetClientRect, GetCursorPos, ClientToScreen, SetCursorPos, ShowWindow,
        SW_MAXIMIZE, POINT,
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
}

pub struct MouseCapture {
    #[cfg(target_os = "linux")]
    inner: Option<x11_capture::X11Capture>,
    #[cfg(windows)]
    inner: Option<win32_capture::Win32Capture>,
    captured: bool,
}

impl MouseCapture {
    pub fn new(window: &Window, width: i32, height: i32) -> MouseCapture {
        #[cfg(target_os = "linux")]
        {
            let inner = x11_capture::X11Capture::new(window, width, height);
            MouseCapture {
                inner,
                captured: false,
            }
        }
        #[cfg(windows)]
        {
            let inner = win32_capture::Win32Capture::new(window, width, height);
            MouseCapture {
                inner,
                captured: false,
            }
        }
        #[cfg(not(any(target_os = "linux", windows)))]
        {
            let _ = (window, width, height);
            MouseCapture { captured: false }
        }
    }

    pub fn has_native_capture(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            self.inner.is_some()
        }
        #[cfg(windows)]
        {
            self.inner.is_some()
        }
        #[cfg(not(any(target_os = "linux", windows)))]
        {
            false
        }
    }

    pub fn set_captured(&mut self, captured: bool) {
        self.captured = captured;
        #[cfg(target_os = "linux")]
        {
            if let Some(inner) = &mut self.inner {
                inner.set_captured(captured);
            }
        }
        #[cfg(windows)]
        {
            if let Some(inner) = &mut self.inner {
                inner.set_captured(captured);
            }
        }
    }

    pub fn is_captured(&self) -> bool {
        self.captured
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        #[cfg(target_os = "linux")]
        {
            if let Some(inner) = &mut self.inner {
                inner.resize(width, height);
            }
        }
        #[cfg(windows)]
        {
            if let Some(inner) = &mut self.inner {
                inner.resize(width, height);
            }
        }
        #[cfg(not(any(target_os = "linux", windows)))]
        {
            let _ = (width, height);
        }
    }

    pub fn poll_delta(&self) -> (f32, f32) {
        #[cfg(target_os = "linux")]
        {
            if let Some(inner) = &self.inner {
                return inner.poll_delta_and_recenter();
            }
            (0.0, 0.0)
        }
        #[cfg(windows)]
        {
            if let Some(inner) = &self.inner {
                return inner.poll_delta_and_recenter();
            }
            (0.0, 0.0)
        }
        #[cfg(not(any(target_os = "linux", windows)))]
        {
            (0.0, 0.0)
        }
    }
}

pub fn maximize_window(window: &Window) {
    #[cfg(target_os = "linux")]
    {
        x11_capture::maximize_x11(window);
    }
    #[cfg(windows)]
    {
        win32_capture::maximize_win32(window);
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = window;
    }
}
