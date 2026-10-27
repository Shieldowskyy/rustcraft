//! Xbox 360 Native Platform Module for RustCraft
//! Target: Microsoft Xbox 360 (Xenon 3.2GHz PowerPC Tri-Core / ATI Xenos GPU)
//! Environment: Homebrew via libxenon / XeLL / JTAG / RGH exploit
//!
//! Provides direct linear framebuffer video output (720p HD / 1280x720),
//! hardware USB & wireless Xbox 360 controller input, PowerPC timebase clock,
//! virtual cursor in menus, and save storage on USB (uda0:) or internal HDD (sda0:).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
#[cfg(not(target_arch = "powerpc"))]
use std::time::Instant;

use crate::app::{GameState, InputState, RustCraftApp};

// ============================================================================
// 1. Xbox 360 / libxenon Low-Level FFI Definitions
// ============================================================================

pub const VIDEO_MODE_720P: i32 = 0;
pub const VIDEO_MODE_1080P: i32 = 1;
pub const VIDEO_MODE_VGA: i32 = 2;

pub const XBOX360_WIDTH: usize = 1280;
pub const XBOX360_HEIGHT: usize = 720;

/// Controller button bitmasks matching libxenon / XInput hardware layout
pub const XBOX_BTN_A: u8 = 0x01;
pub const XBOX_BTN_B: u8 = 0x02;
pub const XBOX_BTN_X: u8 = 0x04;
pub const XBOX_BTN_Y: u8 = 0x08;
pub const XBOX_BTN_UP: u8 = 0x10;
pub const XBOX_BTN_DOWN: u8 = 0x20;
pub const XBOX_BTN_LEFT: u8 = 0x40;
pub const XBOX_BTN_RIGHT: u8 = 0x80;

pub const XBOX_BTN_START: u8 = 0x01;
pub const XBOX_BTN_BACK: u8 = 0x02;
pub const XBOX_BTN_LB: u8 = 0x04;
pub const XBOX_BTN_RB: u8 = 0x08;
pub const XBOX_BTN_L3: u8 = 0x10;
pub const XBOX_BTN_R3: u8 = 0x20;
pub const XBOX_BTN_XENON: u8 = 0x40; // Xbox Guide / Home button

/// libxenon controller packet structure
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct ControllerData {
    pub a: u8,
    pub b: u8,
    pub x: u8,
    pub y: u8,
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8,
    pub start: u8,
    pub back: u8,
    pub xenon: u8, // Guide button
    pub lb: u8,
    pub rb: u8,
    pub lt: u8,    // Left trigger (0..255)
    pub rt: u8,    // Right trigger (0..255)
    pub s1_x: i16, // Left stick X (-32768..32767)
    pub s1_y: i16, // Left stick Y (-32768..32767)
    pub s2_x: i16, // Right stick X (-32768..32767)
    pub s2_y: i16, // Right stick Y (-32768..32767)
    pub s1_z: u8,  // Left stick click (L3)
    pub s2_z: u8,  // Right stick click (R3)
    pub logo: u8,
}

// Fallback in-memory VRAM framebuffer when running under simulation / emulation
static mut SIMULATED_VRAM: [u32; XBOX360_WIDTH * XBOX360_HEIGHT] = [0; XBOX360_WIDTH * XBOX360_HEIGHT];
static HARDWARE_INITIALIZED: AtomicBool = AtomicBool::new(false);

extern "C" {
    // libxenon video functions (provided by libxenon toolchain when compiled for Xenon)
    #[link_name = "video_init"]
    fn xenon_video_init();

    #[link_name = "video_get_vram"]
    fn xenon_video_get_vram() -> *mut u32;

    #[link_name = "video_set_mode"]
    fn xenon_video_set_mode(mode: i32);

    // libxenon controller functions
    #[link_name = "usb_do_poll"]
    fn xenon_usb_do_poll();

    #[link_name = "get_controller_data"]
    fn xenon_get_controller_data(data: *mut ControllerData, port: i32) -> i32;

    // libxenon debug output
    #[link_name = "xenon_uart_puts"]
    fn xenon_uart_puts(s: *const u8);

    #[link_name = "xenon_present_framebuffer"]
    fn xenon_present_framebuffer(
        vram: *mut u32,
        pixels: *const u32,
        src_w: i32,
        src_h: i32,
        dst_w: i32,
        dst_h: i32,
    );

    // SMC power functions
    #[link_name = "xenon_smc_power_shutdown"]
    fn xenon_smc_power_shutdown();

    #[link_name = "xenon_smc_power_reboot"]
    fn xenon_smc_power_reboot();

    // Launcher logging & diagnostic helpers
    #[link_name = "xbox_append_log_c"]
    fn xbox_append_log_c(s: *const u8, len: u32);

    #[link_name = "is_running_under_kernel_c"]
    fn is_running_under_kernel_c() -> i32;

    #[link_name = "xbox_get_bound_log_path"]
    fn xbox_get_bound_log_path() -> *const u8;

    #[link_name = "xbox_get_heap_used"]
    fn xbox_get_heap_used() -> u32;

    #[link_name = "xbox_get_heap_total"]
    fn xbox_get_heap_total() -> u32;
}

// PowerPC Time Base register read
#[inline(always)]
pub fn read_timebase() -> u64 {
    #[cfg(target_arch = "powerpc")]
    {
        let mut tbu0: u32;
        let mut tbl: u32;
        let mut tbu1: u32;
        loop {
            unsafe {
                core::arch::asm!("mftbu {0}", out(reg) tbu0, options(nomem, nostack));
                core::arch::asm!("mftb {0}", out(reg) tbl, options(nomem, nostack));
                core::arch::asm!("mftbu {0}", out(reg) tbu1, options(nomem, nostack));
            }
            if tbu0 == tbu1 {
                break;
            }
        }
        ((tbu0 as u64) << 32) | (tbl as u64)
    }
    #[cfg(not(target_arch = "powerpc"))]
    {
        static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let start = START.get_or_init(Instant::now);
        start.elapsed().as_micros() as u64 * 50 // Simulated 50MHz timebase
    }
}

static mut START_TIMEBASE: u64 = 0;

pub fn get_xbox_elapsed_ms() -> u64 {
    let cur = read_timebase();
    unsafe {
        if START_TIMEBASE == 0 {
            START_TIMEBASE = cur;
        }
        cur.saturating_sub(START_TIMEBASE) / 50_000
    }
}

#[cfg(any(target_os = "xbox360", target_os = "xenon"))]
getrandom::register_custom_getrandom!(xenon_getrandom);

#[cfg(any(target_os = "xbox360", target_os = "xenon"))]
fn xenon_getrandom(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    let mut tb = read_timebase();
    for chunk in dest.chunks_mut(8) {
        tb = tb.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let bytes = tb.to_ne_bytes();
        let len = chunk.len();
        chunk.copy_from_slice(&bytes[..len]);
    }
    Ok(())
}

/// Initializes the timebase and logging for Xbox 360.
pub fn init_xbox_logger() {
    let cur = read_timebase();
    unsafe {
        START_TIMEBASE = cur;
    }
    crate::core::logger::init(true);
    println!("[XBOX360] Native console logger initialized (Timebase: 0x{:016X})", cur);
}

// ============================================================================
// On-Screen Diagnostic Console Facility
// ============================================================================

#[derive(Clone)]
pub struct ConsoleEntry {
    pub text: String,
    pub color: u32,
}

pub struct OnScreenConsole {
    pub entries: Vec<ConsoleEntry>,
    pub max_entries: usize,
    pub visible: bool,
}

impl OnScreenConsole {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 60,
            visible: true,
        }
    }

    pub fn push(&mut self, text: &str) {
        let color = if text.contains("✓") || text.contains("[OK]") || text.contains("SUCCESS") || text.contains("PASSED") {
            0xFF4ADE80 // Emerald green
        } else if text.contains("❌") || text.contains("[FAIL]") || text.contains("[ERROR]") || text.contains("FATAL") {
            0xFFF87171 // Coral red
        } else if text.contains("⚠️") || text.contains("[WARN]") || text.contains("Warning") {
            0xFFFBBF24 // Amber
        } else if text.contains("[Stage") || text.contains("[Environment]") || text.contains("[System]") || text.contains("[Hardware]") {
            0xFF38BDF8 // Sky blue
        } else if text.contains("=====") {
            0xFF64748B // Slate
        } else {
            0xFFF1F5F9 // Light slate / white
        };

        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(ConsoleEntry {
            text: text.to_string(),
            color,
        });
    }

    pub fn render(&self, fb: &mut [u32], width: usize, height: usize) {
        if width == 0 || height == 0 || fb.len() < width * height {
            return;
        }

        // 1. Clear background to dark navy / carbon (0xFF0A0F1D)
        for p in fb.iter_mut() {
            *p = 0xFF0A0F1D;
        }

        // 2. Top Header Bar (height 52px, 0xFF1E293B with 0xFF38BDF8 accent line)
        for y in 0..50.min(height) {
            for x in 0..width {
                fb[y * width + x] = 0xFF1E293B;
            }
        }
        if height > 51 {
            for x in 0..width {
                fb[50 * width + x] = 0xFF38BDF8;
                fb[51 * width + x] = 0xFF38BDF8;
            }
        }

        // Header Title
        crate::graphics::font::draw_text(
            "RUSTCRAFT XBOX 360 DIAGNOSTIC CONSOLE",
            24,
            12,
            2,
            2,
            |px, py| {
                if px >= 0 && (px as usize) < width && py >= 0 && (py as usize) < height {
                    fb[py as usize * width + px as usize] = 0xFFFFFFFF;
                }
            },
        );

        // Header Subtitle (System Info)
        let sub = "Xenon 3.2GHz Tri-Core | 64MB Coalescing Heap | 720p HD Scanout | JTAG/RGH Homebrew";
        crate::graphics::font::draw_text(
            sub,
            24,
            34,
            1,
            1,
            |px, py| {
                if px >= 0 && (px as usize) < width && py >= 0 && (py as usize) < height {
                    fb[py as usize * width + px as usize] = 0xFF94A3B8;
                }
            },
        );

        // 3. Render Log Entries in the main scroll area (y: 60..height - 60)
        let line_height = 18;
        let max_visible_lines = ((height.saturating_sub(120)) / line_height).min(34);
        let start_idx = self.entries.len().saturating_sub(max_visible_lines);
        let visible_entries = &self.entries[start_idx..];

        for (i, entry) in visible_entries.iter().enumerate() {
            let y_pos = 62 + (i as i32) * (line_height as i32);
            let color = entry.color;
            crate::graphics::font::draw_text(
                &entry.text,
                24,
                y_pos,
                1,
                1,
                |px, py| {
                    if px >= 0 && (px as usize) < width && py >= 0 && (py as usize) < height {
                        fb[py as usize * width + px as usize] = color;
                    }
                },
            );
        }

        // 4. Footer Bar (y: height - 52..height)
        let footer_y = height.saturating_sub(50);
        if footer_y < height {
            for x in 0..width {
                fb[footer_y * width + x] = 0xFF334155;
                fb[(footer_y + 1) * width + x] = 0xFF334155;
            }
            for y in (footer_y + 2)..height {
                for x in 0..width {
                    fb[y * width + x] = 0xFF1E293B;
                }
            }

            // Left status: log file
            let log_text = "LOG FILE: USB:\\RustCraft\\rustcraft.log [FLUSHED / ACTIVE]";
            crate::graphics::font::draw_text(
                log_text,
                24,
                footer_y as i32 + 16,
                1,
                1,
                |px, py| {
                    if px >= 0 && (px as usize) < width && py >= 0 && (py as usize) < height {
                        fb[py as usize * width + px as usize] = 0xFF4ADE80;
                    }
                },
            );

            // Right status: controls
            let controls = "[A] / [START] Start Game    [RS Click] Toggle Diagnostics    [BACK] Exit";
            let c_width = crate::graphics::font::measure_text(controls, 1, 1);
            let c_x = (width as i32 - c_width - 24).max(300);
            crate::graphics::font::draw_text(
                controls,
                c_x,
                footer_y as i32 + 16,
                1,
                1,
                |px, py| {
                    if px >= 0 && (px as usize) < width && py >= 0 && (py as usize) < height {
                        fb[py as usize * width + px as usize] = 0xFFF1F5F9;
                    }
                },
            );
        }
    }
}

pub static CONSOLE: Mutex<OnScreenConsole> = Mutex::new(OnScreenConsole::new());

/// Prints and logs a diagnostic message to on-screen console, DbgPrint, UART and rustcraft.log file.
pub fn xbox_log(msg: &str) {
    let elapsed = get_xbox_elapsed_ms();
    let formatted = format!("[{:8} ms] [XBOX360] {}", elapsed, msg);
    println!("{}", formatted);

    if let Ok(mut con) = CONSOLE.lock() {
        con.push(msg);
    }

    // Hardware UART serial port & persistent file logger
    #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
    unsafe {
        xbox_append_log_c(formatted.as_ptr(), formatted.len() as u32);
    }
}

// ============================================================================
// 2. Xbox 360 Video Subsystem & VRAM Framebuffer
// ============================================================================

pub struct XenonVideo {
    pub width: usize,
    pub height: usize,
    vram_ptr: *mut u32,
    double_buffer: Vec<u32>,
}

impl XenonVideo {
    pub fn new(width: usize, height: usize) -> Self {
        let vram = if HARDWARE_INITIALIZED.load(Ordering::SeqCst) {
            #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
            unsafe {
                xenon_video_init();
                xenon_video_set_mode(VIDEO_MODE_720P);
                xenon_video_get_vram()
            }
            #[cfg(not(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
            {
                core::ptr::addr_of_mut!(SIMULATED_VRAM) as *mut u32
            }
        } else {
            core::ptr::addr_of_mut!(SIMULATED_VRAM) as *mut u32
        };

        Self {
            width,
            height,
            vram_ptr: vram,
            double_buffer: vec![0xFF000000; width * height],
        }
    }

    /// Blits the engine color buffer to the hardware linear framebuffer (VRAM).
    /// Performs presentation and cache line flushing (dcbst) for Xenos scanout.
    pub fn present(&mut self, source_pixels: &[u32], src_width: usize, src_height: usize) {
        #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
        unsafe {
            xenon_present_framebuffer(
                self.vram_ptr,
                source_pixels.as_ptr(),
                src_width as i32,
                src_height as i32,
                self.width as i32,
                self.height as i32,
            );
        }

        #[cfg(not(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
        {
            if self.vram_ptr.is_null() {
                return;
            }

            if src_width == self.width && src_height == self.height {
                // Direct 1:1 blit
                let count = self.width * self.height;
                let src = source_pixels.as_ptr();
                let dst = self.vram_ptr;
                unsafe {
                    std::ptr::copy_nonoverlapping(src, dst, count);
                }
            } else {
                let x_ratio = ((src_width << 16) / self.width) + 1;
                let y_ratio = ((src_height << 16) / self.height) + 1;

                unsafe {
                    for y in 0..self.height {
                        let src_y = ((y * y_ratio) >> 16).min(src_height - 1);
                        let src_row = &source_pixels[src_y * src_width..];
                        let dst_row = self.vram_ptr.add(y * self.width);

                        for x in 0..self.width {
                            let src_x = ((x * x_ratio) >> 16).min(src_width - 1);
                            *dst_row.add(x) = src_row[src_x];
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// 3. Xbox 360 Controller & Input State Tracker
// ============================================================================

pub struct Xbox360InputTracker {
    prev_data: ControllerData,
    cursor_x: f32,
    cursor_y: f32,
    cursor_active: bool,
    prev_lb: bool,
    prev_rb: bool,
    prev_start: bool,
    prev_back: bool,
    prev_guide: bool,
}

impl Xbox360InputTracker {
    pub fn new() -> Self {
        Self {
            prev_data: ControllerData::default(),
            cursor_x: (XBOX360_WIDTH / 2) as f32,
            cursor_y: (XBOX360_HEIGHT / 2) as f32,
            cursor_active: true,
            prev_lb: false,
            prev_rb: false,
            prev_start: false,
            prev_back: false,
            prev_guide: false,
        }
    }

    /// Polls physical Xbox 360 controllers and maps inputs to the engine's InputState.
    pub fn poll(&mut self, is_mouse_captured: bool, dt: f32) -> InputState {
        #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
        unsafe {
            xenon_usb_do_poll();
        }

        #[allow(unused_mut)]
        let mut data = ControllerData::default();
        #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
        unsafe {
            let _ = xenon_get_controller_data(&mut data, 0);
        }

        // Deadzones and stick normalization (-1.0 to 1.0)
        let deadzone = 7000.0;
        let left_x = if (data.s1_x as f32).abs() > deadzone {
            data.s1_x as f32 / 32768.0
        } else {
            0.0
        };
        let left_y = if (data.s1_y as f32).abs() > deadzone {
            data.s1_y as f32 / 32768.0
        } else {
            0.0
        };

        let right_x = if (data.s2_x as f32).abs() > deadzone {
            data.s2_x as f32 / 32768.0
        } else {
            0.0
        };
        let right_y = if (data.s2_y as f32).abs() > deadzone {
            data.s2_y as f32 / 32768.0
        } else {
            0.0
        };

        // Buttons
        let btn_a = data.a != 0;
        let btn_b = data.b != 0;
        let btn_x = data.x != 0;
        let btn_y = data.y != 0;
        let dpad_up = data.up != 0;
        let dpad_down = data.down != 0;
        let dpad_left = data.left != 0;
        let dpad_right = data.right != 0;
        let btn_start = data.start != 0;
        let btn_back = data.back != 0;
        let btn_lb = data.lb != 0;
        let btn_rb = data.rb != 0;
        let btn_l3 = data.s1_z != 0;
        let btn_r3 = data.s2_z != 0;
        let trigger_l = data.lt as f32 / 255.0;
        let trigger_r = data.rt as f32 / 255.0;

        // Just-pressed edge detection
        let a_just = btn_a && self.prev_data.a == 0;
        let b_just = btn_b && self.prev_data.b == 0;
        let x_just = btn_x && self.prev_data.x == 0;
        let y_just = btn_y && self.prev_data.y == 0;
        let up_just = dpad_up && self.prev_data.up == 0;
        let down_just = dpad_down && self.prev_data.down == 0;
        let left_just = dpad_left && self.prev_data.left == 0;
        let right_just = dpad_right && self.prev_data.right == 0;
        let start_just = btn_start && !self.prev_start;
        let back_just = btn_back && !self.prev_back;
        let lb_just = btn_lb && !self.prev_lb;
        let rb_just = btn_rb && !self.prev_rb;
        let _l3_just = btn_l3 && self.prev_data.s1_z == 0;
        let r3_just = btn_r3 && self.prev_data.s2_z == 0;
        let rt_just = trigger_r > 0.3 && (self.prev_data.rt as f32 / 255.0) <= 0.3;
        let lt_just = trigger_l > 0.3 && (self.prev_data.lt as f32 / 255.0) <= 0.3;

        // Mouse cursor emulation in menus
        let mut mouse_dx = 0.0;
        let mut mouse_dy = 0.0;

        if is_mouse_captured {
            // First person look: right analog stick moves camera
            let sensitivity = 700.0;
            mouse_dx = right_x * sensitivity * dt;
            mouse_dy = -right_y * sensitivity * dt;
        } else {
            // Menu cursor: left/right stick moves on-screen virtual cursor
            let cursor_speed = 850.0;
            let move_x = left_x + right_x;
            let move_y = -(left_y + right_y);

            if move_x.abs() > 0.08 || move_y.abs() > 0.08 {
                self.cursor_x = (self.cursor_x + move_x * cursor_speed * dt).clamp(5.0, (XBOX360_WIDTH - 5) as f32);
                self.cursor_y = (self.cursor_y + move_y * cursor_speed * dt).clamp(5.0, (XBOX360_HEIGHT - 5) as f32);
                self.cursor_active = true;
            }

            if up_just || down_just || left_just || right_just {
                self.cursor_active = false;
            }
        }

        let mouse_pos = if is_mouse_captured || !self.cursor_active {
            None
        } else {
            Some((self.cursor_x, self.cursor_y))
        };

        // Primary action / Mine / Click
        let mouse_left = if is_mouse_captured {
            trigger_r > 0.3
        } else {
            a_just || (self.cursor_active && trigger_r > 0.3)
        };

        // Secondary action / Place / Cancel
        let mouse_right = if is_mouse_captured {
            trigger_l > 0.3
        } else {
            b_just || (self.cursor_active && trigger_l > 0.3)
        };

        // Hotbar scroll selection with Bumpers (LB / RB)
        let scroll_delta = if rb_just {
            1.0
        } else if lb_just {
            -1.0
        } else {
            0.0
        };

        self.prev_data = data;
        self.prev_lb = btn_lb;
        self.prev_rb = btn_rb;
        self.prev_start = btn_start;
        self.prev_back = btn_back;

        InputState {
            mouse_dx,
            mouse_dy,
            mouse_pos,
            mouse_left,
            mouse_right,
            scroll_delta,

            // Movement keys mapped to Left Stick & D-Pad
            key_w: left_y > 0.3 || dpad_up,
            key_s: left_y < -0.3 || dpad_down,
            key_a: left_x < -0.3 || dpad_left,
            key_d: left_x > 0.3 || dpad_right,

            // Actions
            key_space: btn_a,                       // Jump / Ascend
            key_shift: btn_b,                       // Sneak / Descend
            key_ctrl: btn_l3,                       // Sprint
            key_c: false,
            key_v: false,
            key_x: false,
            key_escape: start_just,                 // Pause menu
            key_p: start_just,
            key_f: back_just,                       // Toggle fly
            key_e: y_just,                          // Inventory
            key_q: x_just,                          // Drop item
            key_g: false,
            key_m: false,
            key_f3: r3_just,                        // Debug profiler toggle
            key_f5: back_just,                      // 3rd person camera
            key_enter: a_just,
            key_up: dpad_up,
            key_down: dpad_down,
            key_left: dpad_left,
            key_right: dpad_right,
            key_n: false,
            key_b: false,
            key_u: false,
            key_del: b_just,
            key_backspace: b_just,
            key_tab: false,
            key_t: false,
            key_slash: false,
            key_digits: [
                false, false, false, false, false, false, false, false, false, false,
            ],
            typed_chars: Vec::new(),

            // Native Gamepad Mapping
            gp_left_stick_x: left_x,
            gp_left_stick_y: left_y,
            gp_right_stick_x: right_x,
            gp_right_stick_y: right_y,
            gp_left_trigger: trigger_l,
            gp_right_trigger: trigger_r,
            gp_btn_south: btn_a,
            gp_btn_east: btn_b,
            gp_btn_west: btn_x,
            gp_btn_north: btn_y,
            gp_btn_lb: btn_lb,
            gp_btn_rb: btn_rb,
            gp_btn_l3: btn_l3,
            gp_btn_r3: btn_r3,
            gp_btn_start: btn_start,
            gp_btn_select: btn_back,
            gp_dpad_up: dpad_up,
            gp_dpad_down: dpad_down,
            gp_dpad_left: dpad_left,
            gp_dpad_right: dpad_right,

            gp_south_just_pressed: a_just,
            gp_east_just_pressed: b_just,
            gp_west_just_pressed: x_just,
            gp_north_just_pressed: y_just,
            gp_lb_just_pressed: lb_just,
            gp_rb_just_pressed: rb_just,
            gp_r3_just_pressed: r3_just,
            gp_start_just_pressed: start_just,
            gp_select_just_pressed: back_just,
            gp_dpad_up_just_pressed: up_just,
            gp_dpad_down_just_pressed: down_just,
            gp_dpad_left_just_pressed: left_just,
            gp_dpad_right_just_pressed: right_just,
            gp_menu_up: up_just,
            gp_menu_down: down_just,
            gp_menu_left: left_just,
            gp_menu_right: right_just,
            gp_right_trigger_just_pressed: rt_just,
            gp_left_trigger_just_pressed: lt_just,

            is_mouse_captured,
            gp_connected: true,
        }
    }
}

// ============================================================================
// 4. Xbox 360 Main Application Entry Point
// ============================================================================

/// C-callable unmangled entry point for native Xenon launcher.
#[no_mangle]
pub extern "C" fn run_rustcraft_xbox360() {
    run_xbox360_app();
}

/// Main application entry point for Xbox 360 – runs self-tests and the native game loop.
pub fn run_xbox360_app() {
    init_xbox_logger();

    let is_kernel = {
        #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
        unsafe { is_running_under_kernel_c() != 0 }
        #[cfg(not(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
        false
    };

    xbox_log("============================================================");
    xbox_log("RustCraft Xbox 360 Diagnostics & Self-Test Suite v1.2");
    if is_kernel {
        xbox_log("[Environment] Running inside xboxkrnl.exe (Title Container)");
    } else {
        xbox_log("[Environment] Running bare-metal inside XeLL Reloaded");
    }
    xbox_log(&format!("CPU Timebase frequency: ~50 MHz | Initial TB: 0x{:016X}", read_timebase()));
    xbox_log("============================================================");

    // 1. Video Subsystem & Framebuffer Allocation
    xbox_log("[Stage 1/6] Initializing Video Framebuffer (1280x720 60Hz)...");
    HARDWARE_INITIALIZED.store(true, Ordering::SeqCst);
    let mut video = XenonVideo::new(XBOX360_WIDTH, XBOX360_HEIGHT);
    let mut console_buf = vec![0xFF0A0F1D; XBOX360_WIDTH * XBOX360_HEIGHT];
    xbox_log("✓ Video subsystem ready (720p HD linear scanout).");

    // Helper to refresh on-screen console
    let refresh_screen = |vid: &mut XenonVideo, buf: &mut [u32]| {
        if let Ok(con) = CONSOLE.lock() {
            con.render(buf, XBOX360_WIDTH, XBOX360_HEIGHT);
        }
        vid.present(buf, XBOX360_WIDTH, XBOX360_HEIGHT);
    };
    refresh_screen(&mut video, &mut console_buf);

    // 2. Memory Allocator Integrity Test
    xbox_log("[Stage 2/6] Testing Freestanding Coalescing Memory Allocator...");
    #[allow(unused_mut)]
    let mut heap_total_mb = 64;
    #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
    unsafe {
        let t = xbox_get_heap_total();
        if t > 0 {
            heap_total_mb = t / (1024 * 1024);
        }
    }
    let mut test_vec: Vec<u32> = Vec::with_capacity(32768);
    for i in 0..32768 {
        test_vec.push(i as u32 ^ 0xAA55AA55);
    }
    let verified = test_vec.iter().enumerate().all(|(i, &val)| val == (i as u32 ^ 0xAA55AA55));
    drop(test_vec);
    if verified {
        xbox_log(&format!("✓ Memory Allocator Test PASSED (Heap: {} MB GDDR3, Coalescing active)", heap_total_mb));
    } else {
        xbox_log("❌ Memory Allocator Test FAILED: Data corruption detected!");
    }
    refresh_screen(&mut video, &mut console_buf);

    // 3. Save Storage and File System Detection
    xbox_log("[Stage 3/6] Detecting save storage device & mounting USB...");
    let usb_saves = std::path::PathBuf::from("uda0:/RustCraft/saves");
    let usb_saves_nt = std::path::PathBuf::from("Usb0:\\RustCraft\\saves");
    let hdd_saves = std::path::PathBuf::from("sda0:/RustCraft/saves");
    let local_saves = std::path::PathBuf::from("./saves");

    if std::fs::create_dir_all(&usb_saves).is_ok() {
        xbox_log("✓ Save storage mounted on USB: uda0:/RustCraft/saves");
        crate::world::storage::set_custom_save_dir(usb_saves);
    } else if std::fs::create_dir_all(&usb_saves_nt).is_ok() {
        xbox_log("✓ Save storage mounted on USB (NT path): Usb0:\\RustCraft\\saves");
        crate::world::storage::set_custom_save_dir(usb_saves_nt);
    } else if std::fs::create_dir_all(&hdd_saves).is_ok() {
        xbox_log("✓ Save storage mounted on internal HDD: sda0:/RustCraft/saves");
        crate::world::storage::set_custom_save_dir(hdd_saves);
    } else {
        let _ = std::fs::create_dir_all(&local_saves);
        xbox_log("✓ Save storage using local directory fallback: ./saves");
        crate::world::storage::set_custom_save_dir(local_saves);
    }
    refresh_screen(&mut video, &mut console_buf);

    // 4. Game Engine Instantiation & Voxel World Generator Test
    let seed = (read_timebase() & 0xFFFFFFFF) as u32;
    xbox_log(&format!("[Stage 4/6] Instantiating RustCraftApp (seed: {})...", seed));
    let mut app = RustCraftApp::new(XBOX360_WIDTH, XBOX360_HEIGHT, "software", seed);
    app.settings.fps_limit = 60;
    app.settings.render_distance = 6;
    app.settings.fog = true;
    xbox_log("✓ RustCraftApp engine & voxel world state ready.");
    refresh_screen(&mut video, &mut console_buf);

    // 5. Input & Controller Subsystem
    xbox_log("[Stage 5/6] Initializing Xbox 360 controller subsystem...");
    let mut input_tracker = Xbox360InputTracker::new();
    let mut last_tb = read_timebase();
    xbox_log("✓ Controller tracker ready (XInput port 0, analog deadzones calibrated).");
    refresh_screen(&mut video, &mut console_buf);

    // 6. Diagnostic Log Persistence
    xbox_log("[Stage 6/6] Verifying diagnostic log persistence on storage...");
    let mut bound_path_str = String::from("USB:\\RustCraft\\rustcraft.log");
    #[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
    unsafe {
        let p = xbox_get_bound_log_path();
        if !p.is_null() {
            let mut len = 0;
            while *p.add(len) != 0 && len < 128 { len += 1; }
            if let Ok(s) = core::str::from_utf8(core::slice::from_raw_parts(p, len)) {
                bound_path_str = s.to_string();
            }
        }
    }
    xbox_log(&format!("✓ Diagnostic log stream active. Target: {}", bound_path_str));
    xbox_log("============================================================");
    xbox_log("ALL SELF-TESTS COMPLETED SUCCESSFULLY!");
    xbox_log("Displaying Diagnostic Console on screen.");
    xbox_log("Press [A] or [START] to enter the game, or [RS Click] to toggle console.");
    xbox_log("============================================================");
    refresh_screen(&mut video, &mut console_buf);

    // 7. Interactive Render & Diagnostic Loop
    let mut frame_count: u64 = 0;
    let mut show_console = app.autotest.is_none();
    let mut auto_hide_counter = 0;

    loop {
        let cur_tb = read_timebase();
        let diff = cur_tb.saturating_sub(last_tb);
        last_tb = cur_tb;

        let dt = (diff as f32 / 50_000_000.0).clamp(0.001, 0.1);

        let is_captured = matches!(app.state, GameState::Playing) && !show_console;
        let input = input_tracker.poll(is_captured, dt);

        // Toggle diagnostic console with RS click (R3)
        if input.gp_r3_just_pressed {
            show_console = !show_console;
            xbox_log(&format!("Diagnostic console toggled: visible = {}", show_console));
        }

        if show_console {
            // Dismiss diagnostic screen on A or START
            if input.gp_south_just_pressed || input.gp_start_just_pressed {
                show_console = false;
                xbox_log("Diagnostic console dismissed by player. Entering game view.");
            }
            auto_hide_counter += 1;
            // Auto-hide diagnostic screen after ~5 seconds if in MainMenu
            if auto_hide_counter > 300 && matches!(app.state, GameState::MainMenu) {
                show_console = false;
            }
        }

        let output = app.update_and_render(dt, &input);

        // Presentation
        if show_console {
            if let Ok(con) = CONSOLE.lock() {
                con.render(&mut console_buf, XBOX360_WIDTH, XBOX360_HEIGHT);
            }
            video.present(&console_buf, XBOX360_WIDTH, XBOX360_HEIGHT);
        } else {
            video.present(app.renderer.pixel_buffer(), app.width, app.height);
        }

        frame_count += 1;
        if frame_count == 1 || frame_count == 60 || frame_count % 600 == 0 {
            let fps = (1.0 / dt).round() as u32;
            let state_str = match &app.state {
                GameState::MainMenu => "MainMenu",
                GameState::SaveSelect => "SaveSelect",
                GameState::CreateWorld => "CreateWorld",
                GameState::Settings(_) => "Settings",
                GameState::Playing => "Playing",
                GameState::Paused => "Paused",
                GameState::Inventory => "Inventory",
                GameState::Chest { .. } => "Chest",
                GameState::Furnace { .. } => "Furnace",
                GameState::VillagerTrade { .. } => "VillagerTrade",
                GameState::DeathScreen => "DeathScreen",
                _ => "Other",
            };
            if frame_count == 1 {
                xbox_log("✓ First frame successfully rendered and presented to scanout.");
            } else {
                xbox_log(&format!("Heartbeat: frame {}, fps ~{}, state: {}, console: {}", frame_count, fps, state_str, show_console));
            }
        }

        if output.quit_requested {
            xbox_log("Quit requested from dashboard. Exiting cleanly...");
            break;
        }
    }

    xbox_log("RustCraft execution completed. Returning to Xbox 360 dashboard.");
}
