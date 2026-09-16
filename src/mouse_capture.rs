// Real FPS-style mouse capture.
//
// `minifb` (used only for the window/input, never for rendering) has no
// built-in way to warp or confine the OS cursor on any platform, so by
// default the cursor can wander off the game window while looking around.
//
// On Linux/X11 we fix this properly: we pull the raw Xlib `Display`/`Window`
// handles out of the `minifb::Window` (via the `raw-window-handle` crate,
// which minifb already implements) and call Xlib directly to:
//   1. Hide the cursor (recolor it fully transparent) while captured.
//   2. Confine pointer motion to the window (`XGrabPointer`).
//   3. After reading the mouse delta each frame, warp the cursor back to
//      the window center (`XWarpPointer`) so it can never reach the edge.
//
// On non-Linux platforms (Windows/macOS) or if X11 setup fails for any
// reason (e.g. running under Wayland without XWayland, or headless), we
// fall back to the previous best-effort approach: track deltas from the
// last observed cursor position and simply accept that the cursor can
// reach the window edge. The game remains fully playable either way.

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

    // Raw Xlib pointers are safe to send between... actually we keep this
    // single-threaded (used only from main.rs's game loop), so we simply
    // avoid Send/Sync entirely by not requiring them.

    impl X11Capture {
        /// Attempts to set up X11 pointer capture for the given minifb
        /// window. Returns None if the window isn't backed by X11 (e.g.
        /// running under pure Wayland) or if Xlib can't be loaded.
        pub fn new(window: &minifb::Window, width: i32, height: i32) -> Option<X11Capture> {
            let display_handle = window.display_handle().ok()?;
            let window_handle = window.window_handle().ok()?;

            let xlib_display = match display_handle.as_raw() {
                RawDisplayHandle::Xlib(h) => h,
                _ => return None, // not X11 (e.g. Wayland) - fall back
            };
            let xlib_window = match window_handle.as_raw() {
                RawWindowHandle::Xlib(h) => h,
                _ => return None,
            };

            let lib = xlib::Xlib::open().ok()?;
            let display = xlib_display.display?.as_ptr() as *mut xlib::Display;
            let xid = xlib_window.window as c_ulong;

            // Build a fully-transparent 1x1 cursor so the real OS cursor is
            // invisible while captured (we draw our own crosshair instead).
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

        /// Reads the current pointer position (relative to the window),
        /// computes the delta from the window center, then warps the
        /// pointer back to the center so it never escapes. Returns
        /// (dx, dy) in pixels, or (0, 0) if not captured or on error.
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

                // Only warp (and thus generate the next frame's delta) if
                // the pointer actually moved, to avoid needless X calls.
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
        // A 1x1 fully-transparent pixmap-based cursor.
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
            longs[0] = 1; // 1 = _NET_WM_STATE_ADD
            longs[1] = max_vert as std::os::raw::c_long;
            longs[2] = max_horz as std::os::raw::c_long;
            longs[3] = 1; // normal application
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

/// Public, cross-platform handle used by main.rs. Wraps the Linux X11
/// implementation when available, otherwise behaves as an inert fallback
/// and the caller keeps using ordinary `get_mouse_pos` delta tracking.
pub struct MouseCapture {
    #[cfg(target_os = "linux")]
    inner: Option<x11_capture::X11Capture>,
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
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (window, width, height);
            MouseCapture { captured: false }
        }
    }

    /// True if real OS-level pointer capture is active (Linux/X11 only).
    /// When false, the caller should fall back to plain delta tracking of
    /// `get_mouse_pos`, and the cursor may be able to reach the window edge.
    pub fn has_native_capture(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            self.inner.is_some()
        }
        #[cfg(not(target_os = "linux"))]
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
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (width, height);
        }
    }

    /// Returns the mouse motion delta since the last call, already
    /// re-centering the OS cursor under the hood when native capture is
    /// active. Returns (0, 0) if not captured.
    pub fn poll_delta(&self) -> (f32, f32) {
        #[cfg(target_os = "linux")]
        {
            if let Some(inner) = &self.inner {
                return inner.poll_delta_and_recenter();
            }
            (0.0, 0.0)
        }
        #[cfg(not(target_os = "linux"))]
        {
            (0.0, 0.0)
        }
    }
}

/// Requests the window manager to maximize the window.
pub fn maximize_window(window: &Window) {
    #[cfg(target_os = "linux")]
    {
        x11_capture::maximize_x11(window);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
    }
}
