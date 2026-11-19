//! Sony PS Vita native on-screen keyboard (`SceImeDialog`) lifecycle manager.
//! Manages dialog state and decodes UTF-16 user input into UTF-8 strings.

#![allow(dead_code)]

use super::ffi::*;

pub struct VitaImeManager {
    pub is_active: bool,
    pub title_utf16: Vec<u16>,
    pub initial_utf16: Vec<u16>,
    pub buffer_utf16: [u16; 128],
}

impl VitaImeManager {
    pub fn new() -> Self {
        Self {
            is_active: false,
            title_utf16: Vec::new(),
            initial_utf16: Vec::new(),
            buffer_utf16: [0; 128],
        }
    }

    pub fn start(&mut self, title: &str, initial_text: &str) -> bool {
        if self.is_active {
            return false;
        }
        self.title_utf16 = title.encode_utf16().chain(core::iter::once(0)).collect();
        self.initial_utf16 = initial_text.encode_utf16().chain(core::iter::once(0)).collect();
        self.buffer_utf16 = [0; 128];

        unsafe {
            let mut param: SceImeDialogParam = core::mem::zeroed();
            sceImeDialogParamInit(&mut param);
            param.supported_languages = SCE_IME_LANGUAGE_ENGLISH;
            param.languages_forced = 1;
            param.dialog_type = SCE_IME_TYPE_DEFAULT;
            param.title = self.title_utf16.as_ptr();
            param.max_text_length = 64;
            param.initial_text = self.initial_utf16.as_ptr();
            param.input_text_buffer = self.buffer_utf16.as_mut_ptr();

            let res = sceImeDialogInit(&param);
            if res >= 0 {
                self.is_active = true;
                crate::vita_log!("SceImeDialog opened for: {}", title);
                true
            } else {
                crate::vita_log!("SceImeDialog failed to initialize (code: {})", res);
                false
            }
        }
    }

    /// Updates the IME dialog state.
    /// NOTE: Passing `NULL` to `sceCommonDialogUpdate` crashes `SceCommonDialog` on PS Vita!
    /// If you have a valid `SceCommonDialogUpdateParam` pointer from `GxmRenderer`, pass it via `update_with_param`.
    pub fn update(&mut self) -> Option<Option<String>> {
        self.update_with_param(core::ptr::null())
    }

    pub fn update_with_param(&mut self, update_param: *const core::ffi::c_void) -> Option<Option<String>> {
        if !self.is_active {
            return None;
        }

        unsafe {
            // Guard against NULL pointer crash in SceCommonDialogUpdate
            if !update_param.is_null() {
                sceCommonDialogUpdate(update_param);
            }

            let status = sceImeDialogGetStatus();
            if status == SCE_COMMON_DIALOG_STATUS_FINISHED {
                let mut result: SceImeDialogResult = core::mem::zeroed();
                sceImeDialogGetResult(&mut result);
                let text_opt = if result.button == SCE_IME_DIALOG_BUTTON_ENTER {
                    let len = self
                        .buffer_utf16
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(self.buffer_utf16.len());
                    String::from_utf16(&self.buffer_utf16[..len]).ok()
                } else {
                    None
                };
                sceImeDialogTerm();
                self.is_active = false;
                Some(text_opt)
            } else if status < 0 || status == SCE_COMMON_DIALOG_STATUS_NONE {
                // Dialog failed or closed unexpectedly
                self.is_active = false;
                Some(None)
            } else {
                None
            }
        }
    }
}

