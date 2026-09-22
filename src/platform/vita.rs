//! PlayStation Vita (PS Vita) Native Platform Module for RustCraft
//! Target: `armv7-sony-vita-newlibeabihf`

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::app::{AppOutput, InputState, RustCraftApp};

// ============================================================================
// 1. VitaSDK Low-Level FFI Definitions (Zero external dependency requirement)
// ============================================================================

pub type SceUID = i32;
pub type SceSize = u32;

pub const SCE_OK: i32 = 0;

// Power & Clocks
#[link(name = "ScePower_stub")]
extern "C" {
    pub fn scePowerSetArmClockFrequency(freq: i32) -> i32;
    pub fn scePowerSetBusClockFrequency(freq: i32) -> i32;
    pub fn scePowerSetGpuClockFrequency(freq: i32) -> i32;
    pub fn scePowerSetGpuXbarClockFrequency(freq: i32) -> i32;
}

// Controller / Pad
pub const SCE_CTRL_MODE_DIGITAL: i32 = 0;
pub const SCE_CTRL_MODE_ANALOG: i32 = 1;
pub const SCE_CTRL_MODE_ANALOG_WIDE: i32 = 2;

pub const SCE_CTRL_SELECT: u32 = 0x00000001;
pub const SCE_CTRL_START: u32 = 0x00000008;
pub const SCE_CTRL_UP: u32 = 0x00000010;
pub const SCE_CTRL_RIGHT: u32 = 0x00000020;
pub const SCE_CTRL_DOWN: u32 = 0x00000040;
pub const SCE_CTRL_LEFT: u32 = 0x00000080;
pub const SCE_CTRL_LTRIGGER: u32 = 0x00000100;
pub const SCE_CTRL_RTRIGGER: u32 = 0x00000200;
pub const SCE_CTRL_TRIANGLE: u32 = 0x00001000;
pub const SCE_CTRL_CIRCLE: u32 = 0x00002000;
pub const SCE_CTRL_CROSS: u32 = 0x00004000;
pub const SCE_CTRL_SQUARE: u32 = 0x00008000;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SceCtrlData {
    pub time_stamp: u64,
    pub buttons: u32,
    pub lx: u8,
    pub ly: u8,
    pub rx: u8,
    pub ry: u8,
    pub up: u8,
    pub right: u8,
    pub down: u8,
    pub left: u8,
    pub lt: u8,
    pub rt: u8,
    pub ppad: [u8; 4],
    pub reserved: [u8; 4],
}

#[link(name = "SceCtrl_stub")]
extern "C" {
    pub fn sceCtrlSetSamplingMode(mode: i32) -> i32;
    pub fn sceCtrlPeekBufferPositive(port: i32, pad_data: *mut SceCtrlData, count: i32) -> i32;
    pub fn sceCtrlReadBufferPositive(port: i32, pad_data: *mut SceCtrlData, count: i32) -> i32;
}

// Touch
pub const SCE_TOUCH_PORT_FRONT: i32 = 0;
pub const SCE_TOUCH_PORT_BACK: i32 = 1;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SceTouchReport {
    pub id: u8,
    pub force: u8,
    pub reserved: [u8; 2],
    pub x: i16,
    pub y: i16,
    pub reserved2: [u8; 8],
    pub info: u16,
}

#[repr(C)]
pub struct SceTouchData {
    pub time_stamp: u64,
    pub status: u32,
    pub report_num: u32,
    pub report: [SceTouchReport; 8],
}

#[link(name = "SceTouch_stub")]
extern "C" {
    pub fn sceTouchSetSamplingState(port: i32, state: i32) -> i32;
    pub fn sceTouchPeek(port: i32, touch_data: *mut SceTouchData, count: i32) -> i32;
}

// Display & Framebuffer
pub const VITA_SCREEN_WIDTH: usize = 960;
pub const VITA_SCREEN_HEIGHT: usize = 544;
pub const VITA_SCREEN_STRIDE: usize = 960;

pub const SCE_DISPLAY_PIXELFORMAT_A8B8G8R8: i32 = 0;
pub const SCE_DISPLAY_SETBUF_IMMEDIATE: i32 = 0;
pub const SCE_DISPLAY_SETBUF_NEXTFRAME: i32 = 1;

#[repr(C)]
pub struct SceDisplayFrameBuf {
    pub size: SceSize,
    pub base: *mut core::ffi::c_void,
    pub pitch: u32,
    pub pixelformat: u32,
    pub width: u32,
    pub height: u32,
}

#[link(name = "SceDisplay_stub")]
extern "C" {
    pub fn sceDisplaySetFrameBuf(
        pParam: *const SceDisplayFrameBuf,
        sync: i32,
    ) -> i32;
    pub fn sceDisplayWaitVblankStart() -> i32;
}

// Kernel & Memory Allocation (CDRAM)
pub const SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW: u32 = 0x09408060;
pub const SCE_KERNEL_MEMBLOCK_TYPE_USER_RW: u32 = 0x0c20d060;

#[link(name = "SceSysmem_stub")]
extern "C" {
    pub fn sceKernelAllocMemBlock(
        name: *const u8,
        type_: u32,
        size: i32,
        optp: *const core::ffi::c_void,
    ) -> SceUID;
    pub fn sceKernelGetMemBlockBase(uid: SceUID, basep: *mut *mut core::ffi::c_void) -> i32;
    pub fn sceKernelFreeMemBlock(uid: SceUID) -> i32;
}

#[link(name = "SceLibKernel_stub")]
extern "C" {
    pub fn sceKernelDelayThread(delay: u32) -> i32;
    pub fn sceKernelGetProcessTimeWide() -> u64;
    pub fn sceKernelExitProcess(res: i32);
    pub fn sceKernelLoadStartModule(
        path: *const u8,
        args: usize,
        argp: *const core::ffi::c_void,
        flags: u32,
        option: *const core::ffi::c_void,
        status: *mut i32,
    ) -> SceUID;
}

// ============================================================================
// 2. Vita Hardware Display & Framebuffer Manager
// ============================================================================

pub struct VitaDisplay {
    memblock_uid: [SceUID; 2],
    framebuffers: [*mut u32; 2],
    current_buffer: usize,
}

impl VitaDisplay {
    pub fn new() -> Result<Self, &'static str> {
        let fb_size = ((VITA_SCREEN_STRIDE * VITA_SCREEN_HEIGHT * 4 + 0xFFFFF) & !0xFFFFF) as i32;
        let mut memblock_uid = [0; 2];
        let mut framebuffers = [core::ptr::null_mut(); 2];

        for i in 0..2 {
            let name = if i == 0 { b"RustCraft_FB0\0".as_ptr() } else { b"RustCraft_FB1\0".as_ptr() };
            // Spróbuj najpierw w pamięci CDRAM (najszybsza pamięć wideo na Vita)
            let mut uid = unsafe {
                sceKernelAllocMemBlock(name, SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, fb_size, core::ptr::null())
            };
            // Fallback do standardowej pamięci RAM jeśli CDRAM byłby zajęty
            if uid < 0 {
                uid = unsafe {
                    sceKernelAllocMemBlock(name, SCE_KERNEL_MEMBLOCK_TYPE_USER_RW, fb_size, core::ptr::null())
                };
            }
            if uid < 0 {
                return Err("Nie udało się zaalokować bufora ramki dla PS Vita");
            }

            let mut base: *mut core::ffi::c_void = core::ptr::null_mut();
            unsafe {
                let res = sceKernelGetMemBlockBase(uid, &mut base);
                if res != SCE_OK || base.is_null() {
                    return Err("Błąd pobierania adresu bazowego bufora ramki");
                }
                // Wyzeruj bufor
                core::ptr::write_bytes(base as *mut u8, 0, (VITA_SCREEN_STRIDE * VITA_SCREEN_HEIGHT * 4) as usize);
            }

            memblock_uid[i] = uid;
            framebuffers[i] = base as *mut u32;
        }

        Ok(Self {
            memblock_uid,
            framebuffers,
            current_buffer: 0,
        })
    }

    /// Kopiuje wyrenderowany bufor gry (960x540) do bufora ramki Vita (960x544) i przełącza bufor
    pub fn present(&mut self, src_pixel_buffer: &[u32], game_width: usize, game_height: usize) {
        let draw_buf = self.framebuffers[self.current_buffer];
        if draw_buf.is_null() {
            return;
        }

        // Kopiowanie linii z uwzględnieniem wyśrodkowania (2 linie marginesu na górze i dole)
        let y_offset = if VITA_SCREEN_HEIGHT > game_height {
            (VITA_SCREEN_HEIGHT - game_height) / 2
        } else {
            0
        };

        let copy_width = game_width.min(VITA_SCREEN_WIDTH);
        let copy_height = game_height.min(VITA_SCREEN_HEIGHT);

        unsafe {
            for y in 0..copy_height {
                let src_offset = y * game_width;
                let dst_offset = (y + y_offset) * VITA_SCREEN_STRIDE;
                let src_ptr = src_pixel_buffer.as_ptr().add(src_offset);
                let dst_ptr = draw_buf.add(dst_offset);
                // Konwersja: gra renderuje 0x00RRGGBB, Vita oczekuje ABGR8888 (0xFFBBGGRR)
                for px in 0..copy_width {
                    let xrgb = *src_ptr.add(px);
                    let r = (xrgb >> 16) & 0xFF;
                    let g = (xrgb >> 8) & 0xFF;
                    let b = xrgb & 0xFF;
                    // ABGR: A=0xFF, B, G, R
                    *dst_ptr.add(px) = (0xFF << 24) | (b << 16) | (g << 8) | r;
                }
            }

            // Ustawienie wyświetlania i VSync
            let fb_struct = SceDisplayFrameBuf {
                size: core::mem::size_of::<SceDisplayFrameBuf>() as SceSize,
                base: draw_buf as *mut core::ffi::c_void,
                pitch: VITA_SCREEN_STRIDE as u32,
                pixelformat: SCE_DISPLAY_PIXELFORMAT_A8B8G8R8 as u32,
                width: VITA_SCREEN_WIDTH as u32,
                height: VITA_SCREEN_HEIGHT as u32,
            };

            sceDisplaySetFrameBuf(&fb_struct, SCE_DISPLAY_SETBUF_NEXTFRAME);
            sceDisplayWaitVblankStart();
        }

        // Przełącz na drugi bufor (Double Buffering)
        self.current_buffer = 1 - self.current_buffer;
    }
}

impl Drop for VitaDisplay {
    fn drop(&mut self) {
        for uid in self.memblock_uid {
            if uid > 0 {
                unsafe { sceKernelFreeMemBlock(uid); }
            }
        }
    }
}

// ============================================================================
// 3. Vita Input & Controls Tracker
// ============================================================================

pub struct VitaInputTracker {
    prev_buttons: u32,
    prev_touch_count: u32,
}

impl VitaInputTracker {
    pub fn new() -> Self {
        // Włącz obsługę gałek analogowych i panelu dotykowego
        unsafe {
            sceCtrlSetSamplingMode(SCE_CTRL_MODE_ANALOG_WIDE);
            sceTouchSetSamplingState(SCE_TOUCH_PORT_FRONT, 1);
        }
        Self {
            prev_buttons: 0,
            prev_touch_count: 0,
        }
    }

    pub fn poll(&mut self, is_mouse_captured: bool) -> InputState {
        let mut pad = SceCtrlData::default();
        unsafe {
            sceCtrlPeekBufferPositive(0, &mut pad, 1);
        }

        let buttons = pad.buttons;
        let prev_buttons = self.prev_buttons;
        let just_pressed = move |mask: u32| -> bool { (buttons & mask != 0) && (prev_buttons & mask == 0) };

        // Gałki analogowe (zakres 0..255, 128 to środek)
        let apply_deadzone = |val: u8| -> f32 {
            let f = (val as f32 - 128.0) / 128.0;
            if f.abs() < 0.18 {
                0.0
            } else {
                f
            }
        };

        let left_x = apply_deadzone(pad.lx);
        let left_y = apply_deadzone(pad.ly);
        let right_x = apply_deadzone(pad.rx);
        let right_y = apply_deadzone(pad.ry);

        // Mapowanie przycisków Vita na akcje
        let btn_cross = buttons & SCE_CTRL_CROSS != 0;
        let btn_circle = buttons & SCE_CTRL_CIRCLE != 0;
        let btn_square = buttons & SCE_CTRL_SQUARE != 0;
        let btn_triangle = buttons & SCE_CTRL_TRIANGLE != 0;
        let btn_l = buttons & SCE_CTRL_LTRIGGER != 0;
        let btn_r = buttons & SCE_CTRL_RTRIGGER != 0;
        let btn_start = buttons & SCE_CTRL_START != 0;
        let btn_select = buttons & SCE_CTRL_SELECT != 0;

        let dpad_up = buttons & SCE_CTRL_UP != 0;
        let dpad_down = buttons & SCE_CTRL_DOWN != 0;
        let dpad_left = buttons & SCE_CTRL_LEFT != 0;
        let dpad_right = buttons & SCE_CTRL_RIGHT != 0;

        // Odczyt ekranu dotykowego
        let mut touch = SceTouchData {
            time_stamp: 0,
            status: 0,
            report_num: 0,
            report: [SceTouchReport::default(); 8],
        };
        unsafe {
            sceTouchPeek(SCE_TOUCH_PORT_FRONT, &mut touch, 1);
        }

        let mut touch_pos: Option<(f32, f32)> = None;
        let mut touch_active = false;
        if touch.report_num > 0 {
            // Współrzędne Vita touch: 0..1919 x 0..1087 -> skalujemy do 960 x 540
            let tx = (touch.report[0].x as f32 / 1919.0) * 960.0;
            let ty = (touch.report[0].y as f32 / 1087.0) * 540.0;
            touch_pos = Some((tx.clamp(0.0, 959.0), ty.clamp(0.0, 539.0)));
            touch_active = true;
        }

        // Ruch kamery z prawej gałki
        let mouse_dx = right_x * 8.0;
        let mouse_dy = right_y * 8.0;

        // Hotbar scroll za pomocą D-Pad lewo/prawo
        let scroll_delta = if just_pressed(SCE_CTRL_RIGHT) {
            1.0
        } else if just_pressed(SCE_CTRL_LEFT) {
            -1.0
        } else {
            0.0
        };

        self.prev_buttons = buttons;
        self.prev_touch_count = touch.report_num;

        InputState {
            mouse_dx,
            mouse_dy,
            mouse_pos: touch_pos,
            mouse_left: btn_r || (touch_active && !is_mouse_captured),
            mouse_right: btn_l,
            scroll_delta,

            key_w: left_y < -0.3 || dpad_up,
            key_s: left_y > 0.3 || dpad_down,
            key_a: left_x < -0.3,
            key_d: left_x > 0.3,
            key_space: btn_cross,
            key_shift: btn_circle,
            key_ctrl: false,
            key_c: false,
            key_escape: btn_start,
            key_p: btn_start,
            key_f: btn_select,
            key_e: btn_triangle,
            key_q: btn_square,
            key_g: false,
            key_m: false,
            key_f5: just_pressed(SCE_CTRL_SQUARE),
            key_enter: just_pressed(SCE_CTRL_CROSS),
            key_up: dpad_up,
            key_down: dpad_down,
            key_left: dpad_left,
            key_right: dpad_right,
            key_n: false,
            key_del: btn_circle,
            key_digits: [
                false, false, false, false, false, false, false, false, false, false,
            ],

            gp_left_stick_x: left_x,
            gp_left_stick_y: left_y,
            gp_right_stick_x: right_x,
            gp_right_stick_y: right_y,
            gp_left_trigger: if btn_l { 1.0 } else { 0.0 },
            gp_right_trigger: if btn_r { 1.0 } else { 0.0 },
            gp_btn_south: btn_cross,
            gp_btn_east: btn_circle,
            gp_btn_west: btn_square,
            gp_btn_north: btn_triangle,
            gp_btn_lb: false,
            gp_btn_rb: false,
            gp_btn_l3: false,
            gp_btn_r3: false,
            gp_btn_start: btn_start,
            gp_btn_select: btn_select,
            gp_dpad_up: dpad_up,
            gp_dpad_down: dpad_down,
            gp_dpad_left: dpad_left,
            gp_dpad_right: dpad_right,

            gp_south_just_pressed: just_pressed(SCE_CTRL_CROSS),
            gp_east_just_pressed: just_pressed(SCE_CTRL_CIRCLE),
            gp_west_just_pressed: just_pressed(SCE_CTRL_SQUARE),
            gp_north_just_pressed: just_pressed(SCE_CTRL_TRIANGLE),
            gp_lb_just_pressed: false,
            gp_rb_just_pressed: false,
            gp_r3_just_pressed: false,
            gp_start_just_pressed: just_pressed(SCE_CTRL_START),
            gp_select_just_pressed: just_pressed(SCE_CTRL_SELECT),
            gp_dpad_up_just_pressed: just_pressed(SCE_CTRL_UP),
            gp_dpad_down_just_pressed: just_pressed(SCE_CTRL_DOWN),
            gp_dpad_left_just_pressed: just_pressed(SCE_CTRL_LEFT),
            gp_dpad_right_just_pressed: just_pressed(SCE_CTRL_RIGHT),
            gp_right_trigger_just_pressed: just_pressed(SCE_CTRL_RTRIGGER),
            gp_left_trigger_just_pressed: just_pressed(SCE_CTRL_LTRIGGER),

            is_mouse_captured,
        }
    }
}

// ============================================================================
// 4. PS Vita Main Loop Entry Point
// ============================================================================

/// Ładuje moduł systemowy PS Vita jeśli jeszcze nie jest załadowany
fn load_module(path: &[u8]) {
    unsafe {
        let mut status: i32 = 0;
        sceKernelLoadStartModule(
            path.as_ptr(),
            0,
            core::ptr::null(),
            0,
            core::ptr::null(),
            &mut status,
        );
    }
}

/// Główna funkcja uruchomieniowa dla PlayStation Vita – używa GXM hardware renderer
pub fn run_vita_app() {
    // 1. Załadowanie wymaganych modułów systemowych Vita
    //    GXM wymaga libSceGxm; libc.suprx potrzebne dla malloc/free w shader patcher
    load_module(b"vs0:sys/external/libSceGxm.suprx\0");
    load_module(b"app0:sce_module/libc.suprx\0");
    load_module(b"app0:sce_module/libfios2.suprx\0");

    // 2. Podkręcenie zegarów PS Vita do maksimum
    //    444MHz ARM, 222MHz BUS, 222MHz GPU, 166MHz XBAR
    unsafe {
        scePowerSetArmClockFrequency(444);
        scePowerSetBusClockFrequency(222);
        scePowerSetGpuClockFrequency(222);
        scePowerSetGpuXbarClockFrequency(166);
    }

    // 3. Inicjalizacja silnika gry (logika + świat)
    let seed = unsafe { sceKernelGetProcessTimeWide() } as u32;
    let mut app = RustCraftApp::new(960, 544, "software", seed);

    // 4. Inicjalizacja GXM Hardware Renderer
    //    Alokuje CDRAM triple-buffering, ShaderPatcher, atlas GPU, render target.
    let mut gxm = crate::platform::vita_gxm::GxmRenderer::new(&app.renderer.textures);

    // 5. Inicjalizacja kontrolera pada
    let mut input_tracker = VitaInputTracker::new();
    let is_captured = true; // Na Vita kamera zawsze aktywna (brak myszy)

    let mut last_time = unsafe { sceKernelGetProcessTimeWide() };

    // Główna pętla gry
    loop {
        let current_time = unsafe { sceKernelGetProcessTimeWide() };
        let dt_us = current_time.saturating_sub(last_time);
        last_time = current_time;
        let dt = (dt_us as f32 / 1_000_000.0).clamp(0.001, 0.1);

        // Odczyt wejścia z kontrolera Vita
        let input = input_tracker.poll(is_captured);

        // Aktualizacja logiki gry (fizyka, kamera, świat, AI)
        let output = app.update(dt, &input);

        if output.quit_requested {
            break;
        }

        // Hardware rendering przez GPU Vita (SGX543MP4)
        // sceGxmBeginScene → ustaw shadery → rysuj chunki → sceGxmEndScene → display swap
        gxm.render_frame(&app.camera, &mut app.world);
        gxm.next_buffer();
    }

    unsafe {
        sceKernelExitProcess(0);
    }
}
