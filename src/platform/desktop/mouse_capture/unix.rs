//! X11 pointer grabbing, recentering, F13..F24 keymap polling, and window maximization.

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::os::raw::{c_char, c_int, c_ulong};
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
        unsafe {
            if captured {
                (self.lib.XSetInputFocus)(
                    self.display,
                    self.window,
                    xlib::RevertToParent,
                    xlib::CurrentTime,
                );
                if !self.grabbed {
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
                }
            } else if self.grabbed {
                (self.lib.XUngrabPointer)(self.display, xlib::CurrentTime);
                (self.lib.XUndefineCursor)(self.display, self.window);
            }
            (self.lib.XFlush)(self.display);
        }
        self.grabbed = captured;
    }

    pub fn focus(&self) {
        unsafe {
            (self.lib.XSetInputFocus)(
                self.display,
                self.window,
                xlib::RevertToParent,
                xlib::CurrentTime,
            );
            (self.lib.XFlush)(self.display);
        }
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

    pub fn poll_pointer_mask(&self) -> u32 {
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
            if ok != 0 {
                mask
            } else {
                0
            }
        }
    }

    pub fn poll_keymap(&self, keys_down: &mut std::collections::HashSet<crate::core::controls::KeyCode>) {
        unsafe {
            let mut keys_return = [0u8; 32];
            (self.lib.XQueryKeymap)(self.display, keys_return.as_mut_ptr() as *mut _);

            for (offset, f_code) in (0xFFCA..=0xFFD5).zip([
                crate::core::controls::KeyCode::F13,
                crate::core::controls::KeyCode::F14,
                crate::core::controls::KeyCode::F15,
                crate::core::controls::KeyCode::F16,
                crate::core::controls::KeyCode::F17,
                crate::core::controls::KeyCode::F18,
                crate::core::controls::KeyCode::F19,
                crate::core::controls::KeyCode::F20,
                crate::core::controls::KeyCode::F21,
                crate::core::controls::KeyCode::F22,
                crate::core::controls::KeyCode::F23,
                crate::core::controls::KeyCode::F24,
            ]) {
                let keycode = (self.lib.XKeysymToKeycode)(self.display, offset);
                if keycode > 0 {
                    let kc = keycode as usize;
                    let byte_idx = kc / 8;
                    let bit_idx = kc % 8;
                    if byte_idx < 32 && (keys_return[byte_idx] & (1 << bit_idx)) != 0 {
                        keys_down.insert(f_code);
                    }
                }
            }
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
    let pixmap = (lib.XCreateBitmapFromData)(display, window, data.as_ptr() as *const c_char, 1, 1);
    let mut color: xlib::XColor = std::mem::zeroed();
    let cursor = (lib.XCreatePixmapCursor)(display, pixmap, pixmap, &mut color, &mut color, 0, 0);
    (lib.XFreePixmap)(display, pixmap);
    cursor
}

pub fn maximize_x11(window: &minifb::Window) {
    let (Ok(display_handle), Ok(window_handle)) = (window.display_handle(), window.window_handle())
    else {
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
        let max_vert = (lib.XInternAtom)(
            display,
            b"_NET_WM_STATE_MAXIMIZED_VERT\0".as_ptr() as *const _,
            xlib::False,
        );
        let max_horz = (lib.XInternAtom)(
            display,
            b"_NET_WM_STATE_MAXIMIZED_HORZ\0".as_ptr() as *const _,
            xlib::False,
        );

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

