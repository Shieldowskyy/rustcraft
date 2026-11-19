//! Cross-platform mouse capture facade (X11 on Linux/FreeBSD, Win32 on Windows, delta fallback on macOS/Wayland).

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod unix;
#[cfg(windows)]
mod windows;

use minifb::Window;

pub struct MouseCapture {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    inner: Option<unix::X11Capture>,
    #[cfg(windows)]
    inner: Option<windows::Win32Capture>,
    captured: bool,
}

impl MouseCapture {
    pub fn new(window: &Window, width: i32, height: i32) -> MouseCapture {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            MouseCapture {
                inner: unix::X11Capture::new(window, width, height),
                captured: false,
            }
        }
        #[cfg(windows)]
        {
            MouseCapture {
                inner: windows::Win32Capture::new(window, width, height),
                captured: false,
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "freebsd", windows)))]
        {
            let _ = (window, width, height);
            MouseCapture { captured: false }
        }
    }

    pub fn has_native_capture(&self) -> bool {
        #[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
        {
            self.inner.is_some()
        }
        #[cfg(not(any(target_os = "linux", target_os = "freebsd", windows)))]
        {
            false
        }
    }

    pub fn set_captured(&mut self, captured: bool) {
        self.captured = captured;
        #[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
        if let Some(inner) = &mut self.inner {
            inner.set_captured(captured);
        }
    }

    pub fn is_captured(&self) -> bool {
        self.captured
    }

    pub fn focus(&self) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        if let Some(inner) = &self.inner {
            inner.focus();
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        #[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
        if let Some(inner) = &mut self.inner {
            inner.resize(width, height);
        }
        #[cfg(not(any(target_os = "linux", target_os = "freebsd", windows)))]
        let _ = (width, height);
    }

    pub fn poll_delta(&self) -> (f32, f32) {
        #[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
        if let Some(inner) = &self.inner {
            return inner.poll_delta_and_recenter();
        }
        (0.0, 0.0)
    }

    pub fn poll_extra_buttons(&self) -> (bool, bool, [bool; 4]) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        if let Some(inner) = &self.inner {
            let mask = inner.poll_pointer_mask();
            let mouse_4 = (mask & (1 << 15)) != 0;
            let mouse_5 = (mask & (1 << 16)) != 0;
            let mouse_extra = [
                (mask & (1 << 17)) != 0,
                (mask & (1 << 18)) != 0,
                (mask & (1 << 19)) != 0,
                (mask & (1 << 20)) != 0,
            ];
            return (mouse_4, mouse_5, mouse_extra);
        }
        (false, false, [false; 4])
    }

    pub fn poll_extra_keys(&self, keys_down: &mut std::collections::HashSet<crate::core::controls::KeyCode>) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        if let Some(inner) = &self.inner {
            inner.poll_keymap(keys_down);
        }
        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        let _ = keys_down;
    }
}

pub fn maximize_window(window: &Window) {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    unix::maximize_x11(window);
    #[cfg(windows)]
    windows::maximize_win32(window);
    #[cfg(not(any(target_os = "linux", target_os = "freebsd", windows)))]
    let _ = window;
}

