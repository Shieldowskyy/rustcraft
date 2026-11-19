//! PS Vita thread/heap linker symbols and low-level VitaSDK FFI declarations.
//! Mutates no state directly; exports `sceUserMainThread*`, `_newlib_heap_size_user`, and C ABI stubs.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// ============================================================================
// 0. PS Vita Thread Stack & Heap Configuration (Prevents Stack/Heap OOM Crash)
// ============================================================================
// Default PS Vita main thread stack is tiny (~256 KB) and default libc heap
// is ~32 MB. Without increasing these symbols, allocating RustCraftApp or
// voxel chunks causes an immediate C2-12828-1 crash at startup.

#[no_mangle]
pub static sceUserMainThreadName: [u8; 10] = *b"RustCraft\0";

#[no_mangle]
pub static sceUserMainThreadPriority: i32 = 0x10000100;

// DLACZEGO: Maska `0x7` pozwala wątkowi głównemu działać na wszystkich dostępnych rdzeniach użytkownika (0, 1 i 2), unikając konfliktów z wątkami meszera w tle.
#[no_mangle]
pub static sceUserMainThreadCpuAffinityMask: i32 = 0x7;

/// 4 MB stack size for the main thread
#[no_mangle]
pub static sceUserMainThreadStackSize: u32 = 4 * 1024 * 1024;

/// 180 MB Newlib heap size for Rust `std::alloc` on `armv7-sony-vita-newlibeabihf`.
// DLACZEGO: VitaSDK Newlib allocator reads `_newlib_heap_size_user` at startup; without 180 MB reserved, RustAlloc panics inside `RustCraftApp::new`.
/// Without this symbol, Newlib defaults to a small heap and panics with:
/// `memory allocation of 125829120 bytes failed` inside `RustCraftApp::new`.
#[no_mangle]
pub static _newlib_heap_size_user: u32 = 180 * 1024 * 1024;

// ============================================================================
// 1. VitaSDK Low-Level FFI Definitions (Zero external dependency requirement)
// ============================================================================

pub type SceUID = i32;
pub type SceSize = u32;

pub const SCE_OK: i32 = 0;

// File I/O (low-level SceIofilemgr for crash-safe logging without heap allocations)
pub const SCE_O_WRONLY: i32 = 0x0002;
pub const SCE_O_CREAT: i32 = 0x0200;
pub const SCE_O_TRUNC: i32 = 0x0400;
pub const SCE_O_APPEND: i32 = 0x0100;

#[link(name = "SceIofilemgr_stub")]
extern "C" {
    pub fn sceIoOpen(file: *const u8, flags: i32, mode: i32) -> SceUID;
    pub fn sceIoWrite(fd: SceUID, data: *const core::ffi::c_void, size: SceSize) -> i32;
    pub fn sceIoClose(fd: SceUID) -> i32;
    pub fn sceIoSync(device: *const u8, flag: u32) -> i32;
}

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

/// Exact 32-byte layout matching `SceCtrlData` from `psp2/ctrl.h`.
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
    pub reserved: [u8; 10],
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
pub const SCE_TOUCH_MAX_REPORT: usize = 64;

/// Packed 16-byte touch report matching `SceTouchReport` from `psp2/touch.h`.
#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct SceTouchReport {
    pub id: u8,
    pub force: u8,
    pub x: i16,
    pub y: i16,
    pub reserved: [u8; 8],
    pub info: u16,
}

/// Exact `SceTouchData` layout from `psp2/touch.h`.
// DLACZEGO: `report` MUST have 64 elements (`SCE_TOUCH_MAX_REPORT`); using fewer causes `sceTouchPeek` to overwrite stack memory.
/// Using 8 elements caused `sceTouchPeek` to overwrite ~896 bytes of the stack!
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SceTouchData {
    pub time_stamp: u64,
    pub status: u32,
    pub report_num: u32,
    pub report: [SceTouchReport; SCE_TOUCH_MAX_REPORT],
}

impl Default for SceTouchData {
    fn default() -> Self {
        Self {
            time_stamp: 0,
            status: 0,
            report_num: 0,
            report: [SceTouchReport::default(); SCE_TOUCH_MAX_REPORT],
        }
    }
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
        size: u32,
        optp: *const core::ffi::c_void,
    ) -> SceUID;
    pub fn sceKernelGetMemBlockBase(uid: SceUID, basep: *mut *mut core::ffi::c_void) -> i32;
    pub fn sceKernelFreeMemBlock(uid: SceUID) -> i32;
}

#[link(name = "SceLibKernel_stub")]
extern "C" {
    pub fn sceKernelDelayThread(delay: u32) -> i32;
    pub fn sceKernelGetProcessTimeWide() -> u64;
    pub fn sceKernelExitProcess(res: i32) -> !;
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
// 1b. Sony PS Vita IME (On-Screen Keyboard) Dialog System
// ============================================================================

#[repr(C)]
pub struct SceCommonDialogParam {
    pub infobar_param: *const core::ffi::c_void,
    pub bg_color: *const core::ffi::c_void,
    pub dimmer_color: *const core::ffi::c_void,
    pub reserved: [u8; 60],
    pub magic: u32,
}

pub const SCE_COMMON_DIALOG_MAGIC_NUMBER: u32 = 0xC0D1A109;
pub const PSP2_SDK_VERSION: u32 = 0x03570011;

pub const SCE_IME_TYPE_DEFAULT: u32 = 0;
pub const SCE_IME_LANGUAGE_ENGLISH: u64 = 0x0000000000000002;
pub const SCE_COMMON_DIALOG_STATUS_NONE: i32 = 0;
pub const SCE_COMMON_DIALOG_STATUS_RUNNING: i32 = 1;
pub const SCE_COMMON_DIALOG_STATUS_FINISHED: i32 = 2;
pub const SCE_IME_DIALOG_BUTTON_ENTER: i32 = 0;
pub const SCE_IME_DIALOG_BUTTON_CLOSE: i32 = 1;

#[repr(C)]
pub struct SceImeDialogParam {
    pub sdk_version: u32,
    pub input_method: u32,
    pub supported_languages: u64,
    pub languages_forced: i32,
    pub dialog_type: u32,
    pub option: u32,
    pub filter: *const core::ffi::c_void,
    pub dialog_mode: u32,
    pub text_box_mode: u32,
    pub title: *const u16,
    pub max_text_length: u32,
    pub initial_text: *const u16,
    pub input_text_buffer: *mut u16,
    pub common_param: SceCommonDialogParam,
    pub enter_label: u8,
    pub reserved: [u8; 35],
}

#[inline]
pub unsafe fn sceImeDialogParamInit(param: *mut SceImeDialogParam) {
    core::ptr::write_bytes(param, 0, 1);
    let common_ptr = core::ptr::addr_of_mut!((*param).common_param);
    (*common_ptr).magic = SCE_COMMON_DIALOG_MAGIC_NUMBER.wrapping_add(common_ptr as usize as u32);
    (*param).sdk_version = PSP2_SDK_VERSION;
}

#[repr(C)]
pub struct SceImeDialogResult {
    pub result: i32,
    pub button: i32,
    pub reserved: [u8; 28],
}

#[link(name = "SceIme_stub")]
#[link(name = "SceCommonDialog_stub")]
extern "C" {
    pub fn sceImeDialogInit(param: *const SceImeDialogParam) -> i32;
    pub fn sceImeDialogGetStatus() -> i32;
    pub fn sceImeDialogGetResult(result: *mut SceImeDialogResult) -> i32;
    pub fn sceImeDialogTerm() -> i32;
    pub fn sceCommonDialogUpdate(param: *const core::ffi::c_void) -> i32;
}

