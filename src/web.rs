//! WebAssembly and HTML5 Canvas frontend bindings for RustCraft.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::Clamped;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[cfg(target_arch = "wasm32")]
use crate::app::{InputState, RustCraftApp, HEIGHT, WIDTH};

/// WebAssembly entry point wrapping `RustCraftApp` for browser execution.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebGame {
    app: RustCraftApp,
    ctx: CanvasRenderingContext2d,
    image_data: ImageData,
    rgba_bytes: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebGame {
    /// Creates a new WebAssembly game instance attached to the given `<canvas>` ID.
    ///
    /// # Arguments
    /// * `canvas_id` - DOM element ID of the HTML5 canvas.
    /// * `seed` - Optional world generator seed (defaults to 1337).
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, seed: Option<u32>) -> Result<WebGame, JsValue> {
        console_error_panic_hook::set_once();

        let window = web_sys::window().ok_or_else(|| JsValue::from_str("Missing global window object"))?;
        let document = window.document().ok_or_else(|| JsValue::from_str("Missing document object"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("Canvas element not found with ID '{canvas_id}'")))?;
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;

        canvas.set_width(WIDTH as u32);
        canvas.set_height(HEIGHT as u32);

        let ctx = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2D rendering context from canvas"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let initial_seed = seed.unwrap_or(1337);
        let app = RustCraftApp::new(WIDTH, HEIGHT, "software", initial_seed);
        let rgba_bytes = vec![255u8; WIDTH * HEIGHT * 4];

        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&rgba_bytes),
            WIDTH as u32,
            HEIGHT as u32,
        )?;

        Ok(WebGame {
            app,
            ctx,
            image_data,
            rgba_bytes,
        })
    }

    pub fn get_width(&self) -> usize {
        WIDTH
    }

    pub fn get_height(&self) -> usize {
        HEIGHT
    }

    pub fn get_rgba_ptr(&self) -> *const u8 {
        self.rgba_bytes.as_ptr()
    }

    pub fn update_and_render(
        &mut self,
        dt: f32,
        mouse_dx: f32,
        mouse_dy: f32,
        mouse_x: f32,
        mouse_y: f32,
        mouse_valid: bool,
        mouse_left: bool,
        mouse_right: bool,
        scroll_delta: f32,
        is_pointer_locked: bool,
        key_w: bool,
        key_s: bool,
        key_a: bool,
        key_d: bool,
        key_space: bool,
        key_shift: bool,
        key_ctrl: bool,
        key_c: bool,
        key_escape: bool,
        key_p: bool,
        key_f: bool,
        key_e: bool,
        key_q: bool,
        key_g: bool,
        key_m: bool,
        key_f5: bool,
        key_enter: bool,
        key_up: bool,
        key_down: bool,
        key_left: bool,
        key_right_key: bool,
        key_n: bool,
        key_del: bool,

        key_digit_1: bool,
        key_digit_2: bool,
        key_digit_3: bool,
        key_digit_4: bool,
        key_digit_5: bool,
        key_digit_6: bool,
        key_digit_7: bool,
        key_digit_8: bool,
        key_digit_9: bool,
        key_digit_0: bool,
        gp_lx: f32,
        gp_ly: f32,
        gp_rx: f32,
        gp_ry: f32,
        gp_lt: f32,
        gp_rt: f32,
        gp_btn_south: bool,
        gp_btn_east: bool,
        gp_btn_west: bool,
        gp_btn_north: bool,
        gp_btn_lb: bool,
        gp_btn_rb: bool,
        gp_btn_l3: bool,
        gp_btn_r3: bool,
        gp_btn_start: bool,
        gp_btn_select: bool,
        gp_dpad_up: bool,
        gp_dpad_down: bool,
        gp_dpad_left: bool,
        gp_dpad_right: bool,
        gp_south_just: bool,
        gp_east_just: bool,
        gp_west_just: bool,
        gp_north_just: bool,
        gp_lb_just: bool,
        gp_rb_just: bool,
        gp_r3_just: bool,
        gp_start_just: bool,
        gp_select_just: bool,
        gp_dpad_up_just: bool,
        gp_dpad_down_just: bool,
        gp_dpad_left_just: bool,
        gp_dpad_right_just: bool,
        gp_rt_just: bool,
        gp_lt_just: bool,
    ) -> Result<String, JsValue> {
        let mouse_pos = if mouse_valid {
            Some((mouse_x, mouse_y))
        } else {
            None
        };

        let input = InputState {
            mouse_dx,
            mouse_dy,
            mouse_pos,
            mouse_left,
            mouse_right,
            scroll_delta,

            key_w,
            key_s,
            key_a,
            key_d,
            key_space,
            key_shift,
            key_ctrl,
            key_c,
            key_v: false,
            key_x: false,
            key_escape,
            key_p,
            key_f,
            key_e,
            key_q,
            key_g,
            key_m,
            key_f5,
            key_enter,
            key_up,
            key_down,
            key_left,
            key_right: key_right_key,
            key_n,
            key_del,
            key_backspace: false,
            key_tab: false,
            key_digits: [
                key_digit_1,
                key_digit_2,
                key_digit_3,
                key_digit_4,
                key_digit_5,
                key_digit_6,
                key_digit_7,
                key_digit_8,
                key_digit_9,
                key_digit_0,
            ],
            typed_chars: Vec::new(),

            gp_left_stick_x: gp_lx,
            gp_left_stick_y: gp_ly,
            gp_right_stick_x: gp_rx,
            gp_right_stick_y: gp_ry,
            gp_left_trigger: gp_lt,
            gp_right_trigger: gp_rt,
            gp_btn_south,
            gp_btn_east,
            gp_btn_west,
            gp_btn_north,
            gp_btn_lb,
            gp_btn_rb,
            gp_btn_l3,
            gp_btn_r3,
            gp_btn_start,
            gp_btn_select,
            gp_dpad_up,
            gp_dpad_down,
            gp_dpad_left,
            gp_dpad_right,

            gp_south_just_pressed: gp_south_just,
            gp_east_just_pressed: gp_east_just,
            gp_west_just_pressed: gp_west_just,
            gp_north_just_pressed: gp_north_just,
            gp_lb_just_pressed: gp_lb_just,
            gp_rb_just_pressed: gp_rb_just,
            gp_r3_just_pressed: gp_r3_just,
            gp_start_just_pressed: gp_start_just,
            gp_select_just_pressed: gp_select_just,
            gp_dpad_up_just_pressed: gp_dpad_up_just,
            gp_dpad_down_just_pressed: gp_dpad_down_just,
            gp_dpad_left_just_pressed: gp_dpad_left_just,
            gp_dpad_right_just_pressed: gp_dpad_right_just,
            gp_right_trigger_just_pressed: gp_rt_just,
            gp_left_trigger_just_pressed: gp_lt_just,

            is_mouse_captured: is_pointer_locked,
        };

        let output = self.app.update_and_render(dt, &input);

        // Convert color buffer (0x00RRGGBB) to Canvas ImageData format (RGBA)
        let pixel_buf = self.app.pixel_buffer();
        for (i, &p) in pixel_buf.iter().enumerate() {
            let r = ((p >> 16) & 0xFF) as u8;
            let g = ((p >> 8) & 0xFF) as u8;
            let b = (p & 0xFF) as u8;
            let idx = i * 4;
            self.rgba_bytes[idx] = r;
            self.rgba_bytes[idx + 1] = g;
            self.rgba_bytes[idx + 2] = b;
            self.rgba_bytes[idx + 3] = 255;
        }

        let clamped = Clamped(&self.rgba_bytes[..]);
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            clamped,
            WIDTH as u32,
            HEIGHT as u32,
        )?;
        self.ctx.put_image_data(&image_data, 0.0, 0.0)?;

        let mut actions = Vec::new();
        if output.request_capture_mouse {
            actions.push("capture_mouse");
        }
        if output.request_release_mouse {
            actions.push("release_mouse");
        }
        if output.is_playing {
            actions.push("playing");
        }

        Ok(format!("{}:{}", output.title, actions.join(",")))
    }
}
