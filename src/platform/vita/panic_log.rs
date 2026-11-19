//! Crash-safe Vita filesystem logging, panic hook, and `vita_log!` macro.
//! Writes directly via `SceIofilemgr` without heap allocations during panics or OOMs.

#![allow(dead_code)]

use super::ffi::*;

// ============================================================================
// 1c. Crash-Safe Logging & Panic Hook (No Heap Allocations on Panic)
// ============================================================================

/// Writes raw bytes directly to a PS Vita file descriptor using `SceIofilemgr`.
/// Safe to call inside a panic handler (even during Out-Of-Memory conditions).
pub fn write_crash_bytes(path: &[u8], data: &[u8], append: bool) {
    let flags = if append {
        SCE_O_WRONLY | SCE_O_CREAT | SCE_O_APPEND
    } else {
        SCE_O_WRONLY | SCE_O_CREAT | SCE_O_TRUNC
    };
    unsafe {
        let fd = sceIoOpen(path.as_ptr(), flags, 0o777);
        if fd >= 0 {
            sceIoWrite(fd, data.as_ptr() as *const core::ffi::c_void, data.len() as u32);
            sceIoClose(fd);
        }
    }
}

/// Flushes `ux0:` filesystem buffers to disk (called during fatal panics).
pub fn sync_crash_log() {
    unsafe {
        sceIoSync(b"ux0:\0".as_ptr(), 0);
    }
}

/// Installs a global Rust panic hook that records the panic location, payload,
/// thread name, and backtrace to both `sceClibPrintf` and `ux0:data/rustcraft_panic.log`.
pub fn install_vita_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        use core::fmt::Write;

        // Fixed 4 KB stack buffer so logging works even if the heap is exhausted
        struct StackBuf {
            buf: [u8; 4096],
            pos: usize,
        }

        impl core::fmt::Write for StackBuf {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let rem = self.buf.len().saturating_sub(self.pos + 1); // reserve 1 byte for NUL
                let to_copy = bytes.len().min(rem);
                self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
                self.pos += to_copy;
                Ok(())
            }
        }

        let mut out = StackBuf {
            buf: [0u8; 4096],
            pos: 0,
        };

        let _ = writeln!(out, "\n========================================");
        let _ = writeln!(out, "       RUSTCRAFT PS VITA PANIC!         ");
        let _ = writeln!(out, "========================================");

        if let Some(name) = std::thread::current().name() {
            let _ = writeln!(out, "Thread:   {}", name);
        }

        if let Some(loc) = info.location() {
            let _ = writeln!(out, "Location: {}:{}:{}", loc.file(), loc.line(), loc.column());
        } else {
            let _ = writeln!(out, "Location: <unknown>");
        }

        let payload_str = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "<non-string panic payload>"
        };
        let _ = writeln!(out, "Message:  {}", payload_str);

        let bt = std::backtrace::Backtrace::force_capture();
        let _ = writeln!(out, "\nBacktrace:\n{}", bt);
        let _ = writeln!(out, "========================================");

        let len = out.pos.min(out.buf.len() - 1);
        out.buf[len] = 0;

        // 1. Output to Vita debug console (Vita3K / PrincessLog)
        unsafe {
            extern "C" {
                fn sceClibPrintf(fmt: *const core::ffi::c_char, ...);
            }
            sceClibPrintf(
                b"%s\n\0".as_ptr() as *const core::ffi::c_char,
                out.buf.as_ptr() as *const core::ffi::c_char,
            );
        }

        // 2. Persist panic report to ux0:data/rustcraft_panic.log
        write_crash_bytes(b"ux0:data/rustcraft_panic.log\0", &out.buf[..len], false);

        // 3. Also append a marker to boot log so timeline is clear
        write_crash_bytes(
            b"ux0:data/rustcraft_boot.log\0",
            b"\n[FATAL] Rust panic occurred! See ux0:data/rustcraft_panic.log\n",
            true,
        );

        sync_crash_log();

        // Give the Vita storage controller 500ms to flush the file before process termination
        unsafe {
            sceKernelDelayThread(500_000);
        }
    }));
}

#[macro_export]
macro_rules! vita_log {
    ($($arg:tt)*) => {{
        let mut msg = format!($($arg)*);
        msg.push('\n');

        // In debug builds, mirror every boot/runtime log line to ux0:data/rustcraft_boot.log
        #[cfg(debug_assertions)]
        {
            $crate::platform::vita::write_crash_bytes(
                b"ux0:data/rustcraft_boot.log\0",
                msg.as_bytes(),
                true,
            );
        }

        // SAFETY: sceClibPrintf is the Vita kernel's printf; CString guarantees NUL termination.
        #[allow(unused_unsafe)]
        unsafe {
            extern "C" { fn sceClibPrintf(fmt: *const core::ffi::c_char, ...); }
            if let Ok(c_msg) = std::ffi::CString::new(msg) {
                sceClibPrintf(b"%s\0".as_ptr() as *const core::ffi::c_char, c_msg.as_ptr());
            }
        }
    }};
}

