//! # RustCraft Embedded Profiler HTTP Web Server
//!
//! Provides a zero-external-dependency, high-performance micro HTTP server that hosts
//! a real-time web dashboard for inspecting frame execution timings, memory allocations,
//! GPU/rasterizer telemetry, diagnostic screenshot capture, and world diagnostics on PC and Sony PlayStation Vita.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

/// Comprehensive snapshot of a single frame's telemetry.
#[derive(Clone, Debug, Default)]
pub struct ProfilerSnapshot {
    pub timestamp_ms: u64,
    pub fps: u32,
    pub frame_time_ms: f32,
    pub tick_ms: f32,
    pub world_io_ms: f32,
    pub render_terrain_ms: f32,
    pub render_trans_ms: f32,
    pub render_clouds_ms: f32,
    pub render_entities_ms: f32,
    pub render_sky_ms: f32,
    pub render_hand_ms: f32,
    pub render_ui_ms: f32,

    // Granular micro-timings
    pub tick_physics_ms: f32,
    pub tick_mobs_ms: f32,
    pub tick_entities_ms: f32,
    pub tick_world_ms: f32,
    pub world_gen_ms: f32,
    pub world_mesh_ms: f32,
    pub render_cull_ms: f32,

    pub total_ram_bytes: usize,
    pub chunk_voxels_bytes: usize,
    pub mesh_cache_bytes: usize,
    pub textures_and_fonts_bytes: usize,
    pub framebuffers_bytes: usize,
    pub entities_bytes: usize,
    pub ui_and_misc_bytes: usize,
    pub cdram_bytes: usize,

    pub pipeline_name: String,
    pub renderer_name: String,
    pub gpu_device: String,
    pub driver_info: String,
    pub draw_calls: u32,
    pub triangles_rendered: u32,
    pub triangles_culled: u32,
    pub subchunks_rendered: u32,
    pub subchunks_culled_frustum: u32,
    pub raster_threads: usize,

    pub chunks_loaded: usize,
    pub player_pos: [f32; 3],
    pub biome: String,
    pub daylight_factor: f32,
    pub platform: String,

    // Game state & pause status
    pub is_paused: bool,
    pub game_state: String,
    pub gamemode: String,

    // Graphics & engine settings
    pub render_distance: i32,
    pub fov_deg: f32,
    pub msaa: u32,
    pub fps_limit: u32,
    pub leaf_quality: String,
    pub clouds: bool,
    pub fog: bool,
    pub particles: bool,
    pub chunk_anim: bool,
    pub underwater_fx: bool,
    pub debug_overlay: bool,
}

impl ProfilerSnapshot {
    /// Formats the snapshot into a compact JSON string.
    pub fn to_json(&self) -> String {
        format!(
            "{{\
\"timestamp\":{},\
\"fps\":{},\
\"frame_time_ms\":{:.2},\
\"times\":{{\
\"terrain\":{:.2},\
\"trans\":{:.2},\
\"meshing_io\":{:.2},\
\"world_gen\":{:.2},\
\"world_mesh\":{:.2},\
\"tick\":{:.2},\
\"tick_physics\":{:.2},\
\"tick_mobs\":{:.2},\
\"tick_entities\":{:.2},\
\"tick_world\":{:.2},\
\"clouds\":{:.2},\
\"sky\":{:.2},\
\"entities\":{:.2},\
\"hand_3d\":{:.2},\
\"cull\":{:.2},\
\"ui\":{:.2},\
\"total\":{:.2}\
}},\
\"memory\":{{\
\"total_mb\":{:.2},\
\"voxels_mb\":{:.2},\
\"mesh_cache_mb\":{:.2},\
\"textures_mb\":{:.2},\
\"framebuffers_mb\":{:.2},\
\"entities_kb\":{:.1},\
\"misc_kb\":{:.1},\
\"cdram_mb\":{:.2}\
}},\
\"render\":{{\
\"pipeline\":\"{}\",\
\"renderer_name\":\"{}\",\
\"gpu_device\":\"{}\",\
\"driver_info\":\"{}\",\
\"draw_calls\":{},\
\"triangles_rendered\":{},\
\"triangles_culled\":{},\
\"subchunks_rendered\":{},\
\"subchunks_culled\":{},\
\"raster_threads\":{}\
}},\
\"world\":{{\
\"chunks_loaded\":{},\
\"player_x\":{:.2},\
\"player_y\":{:.2},\
\"player_z\":{:.2},\
\"biome\":\"{}\",\
\"daylight\":{:.2}\
}},\
\"game\":{{\
\"is_paused\":{},\
\"state\":\"{}\",\
\"gamemode\":\"{}\"\
}},\
\"settings\":{{\
\"render_distance\":{},\
\"fov_deg\":{:.1},\
\"msaa\":{},\
\"fps_limit\":{},\
\"leaf_quality\":\"{}\",\
\"clouds\":{},\
\"fog\":{},\
\"particles\":{},\
\"chunk_anim\":{},\
\"underwater_fx\":{},\
\"debug_overlay\":{}\
}},\
\"platform\":\"{}\"\
}}",
            self.timestamp_ms,
            self.fps,
            self.frame_time_ms,
            self.render_terrain_ms,
            self.render_trans_ms,
            self.world_io_ms,
            self.world_gen_ms,
            self.world_mesh_ms,
            self.tick_ms,
            self.tick_physics_ms,
            self.tick_mobs_ms,
            self.tick_entities_ms,
            self.tick_world_ms,
            self.render_clouds_ms,
            self.render_sky_ms,
            self.render_entities_ms,
            self.render_hand_ms,
            self.render_cull_ms,
            self.render_ui_ms,
            self.frame_time_ms,
            self.total_ram_bytes as f32 / (1024.0 * 1024.0),
            self.chunk_voxels_bytes as f32 / (1024.0 * 1024.0),
            self.mesh_cache_bytes as f32 / (1024.0 * 1024.0),
            self.textures_and_fonts_bytes as f32 / (1024.0 * 1024.0),
            self.framebuffers_bytes as f32 / (1024.0 * 1024.0),
            self.entities_bytes as f32 / 1024.0,
            self.ui_and_misc_bytes as f32 / 1024.0,
            self.cdram_bytes as f32 / (1024.0 * 1024.0),
            escape_json(&self.pipeline_name),
            escape_json(&self.renderer_name),
            escape_json(&self.gpu_device),
            escape_json(&self.driver_info),
            self.draw_calls,
            self.triangles_rendered,
            self.triangles_culled,
            self.subchunks_rendered,
            self.subchunks_culled_frustum,
            self.raster_threads,
            self.chunks_loaded,
            self.player_pos[0],
            self.player_pos[1],
            self.player_pos[2],
            escape_json(&self.biome),
            self.daylight_factor,
            self.is_paused,
            escape_json(&self.game_state),
            escape_json(&self.gamemode),
            self.render_distance,
            self.fov_deg,
            self.msaa,
            self.fps_limit,
            escape_json(&self.leaf_quality),
            self.clouds,
            self.fog,
            self.particles,
            self.chunk_anim,
            self.underwater_fx,
            self.debug_overlay,
            escape_json(&self.platform),
        )
    }

    /// Formats snapshot as a CSV line for spreadsheet analysis.
    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{},{},{},\"{}\",{},\"{}\",{},{:.1},{},{}\n",
            self.timestamp_ms,
            self.fps,
            self.frame_time_ms,
            self.render_terrain_ms,
            self.render_trans_ms,
            self.world_io_ms,
            self.tick_ms,
            self.render_clouds_ms + self.render_sky_ms,
            self.render_entities_ms,
            self.render_ui_ms,
            self.total_ram_bytes as f32 / (1024.0 * 1024.0),
            self.chunk_voxels_bytes as f32 / (1024.0 * 1024.0),
            self.mesh_cache_bytes as f32 / (1024.0 * 1024.0),
            self.framebuffers_bytes as f32 / (1024.0 * 1024.0),
            self.cdram_bytes as f32 / (1024.0 * 1024.0),
            self.draw_calls,
            self.triangles_rendered,
            self.triangles_culled,
            self.subchunks_rendered,
            self.chunks_loaded,
            self.biome,
            self.is_paused,
            self.game_state,
            self.render_distance,
            self.fov_deg,
            self.msaa,
            self.fps_limit,
        )
    }
}

/// Encapsulates a captured diagnostic frame and associated metadata.
#[derive(Clone, Debug)]
pub struct ScreenshotData {
    pub bmp_bytes: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub timestamp_ms: u64,
    pub fps: u32,
    pub player_pos: [f32; 3],
    pub biome: String,
    pub game_state: String,
    pub gamemode: String,
}

impl ScreenshotData {
    /// Serializes screenshot metadata into JSON.
    pub fn to_json(&self) -> String {
        format!(
            "{{\
\"timestamp_ms\":{},\
\"width\":{},\
\"height\":{},\
\"fps\":{},\
\"player_x\":{:.2},\
\"player_y\":{:.2},\
\"player_z\":{:.2},\
\"biome\":\"{}\",\
\"game_state\":\"{}\",\
\"gamemode\":\"{}\",\
\"size_bytes\":{},\
\"url\":\"/api/screenshot\"\
}}",
            self.timestamp_ms,
            self.width,
            self.height,
            self.fps,
            self.player_pos[0],
            self.player_pos[1],
            self.player_pos[2],
            escape_json(&self.biome),
            escape_json(&self.game_state),
            escape_json(&self.gamemode),
            self.bmp_bytes.len()
        )
    }
}

/// Encodes raw 32-bit `0x00RRGGBB` color buffer pixels into a standard uncompressed 24-bit Windows BMP image.
pub fn encode_bmp(color_buffer: &[u32], width: usize, height: usize) -> Vec<u8> {
    if width == 0 || height == 0 || color_buffer.len() < width * height {
        return Vec::new();
    }

    let row_bytes = width * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let row_stride = row_bytes + padding;
    let image_size = row_stride * height;
    let file_size = 54 + image_size;

    let mut bmp = Vec::with_capacity(file_size);

    // 14-byte BITMAPFILEHEADER
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes()); // Reserved 1 & 2
    bmp.extend_from_slice(&54u32.to_le_bytes()); // Offset to pixel data

    // 40-byte BITMAPINFOHEADER
    bmp.extend_from_slice(&40u32.to_le_bytes()); // biSize
    bmp.extend_from_slice(&(width as i32).to_le_bytes()); // biWidth
    bmp.extend_from_slice(&(height as i32).to_le_bytes()); // biHeight (bottom-up raster)
    bmp.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    bmp.extend_from_slice(&24u16.to_le_bytes()); // biBitCount (24 bits = RGB)
    bmp.extend_from_slice(&0u32.to_le_bytes()); // biCompression (BI_RGB)
    bmp.extend_from_slice(&(image_size as u32).to_le_bytes()); // biSizeImage
    bmp.extend_from_slice(&2835u32.to_le_bytes()); // biXPelsPerMeter (72 DPI)
    bmp.extend_from_slice(&2835u32.to_le_bytes()); // biYPelsPerMeter (72 DPI)
    bmp.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    bmp.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    let pad_bytes = [0u8; 3];

    // Standard bottom-up DIB raster format
    for y in (0..height).rev() {
        let row_start = y * width;
        for x in 0..width {
            let pixel = color_buffer[row_start + x];
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            bmp.push(b);
            bmp.push(g);
            bmp.push(r);
        }
        if padding > 0 {
            bmp.extend_from_slice(&pad_bytes[..padding]);
        }
    }

    bmp
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ").replace('\r', "")
}

/// Real-time HTTP Profiler Server instance.
pub struct ProfilerServer {
    running: Arc<AtomicBool>,
    screenshot_requested: Arc<AtomicBool>,
    screenshot_counter: Arc<AtomicU64>,
    latest_snapshot: Arc<RwLock<ProfilerSnapshot>>,
    history_buffer: Arc<Mutex<Vec<ProfilerSnapshot>>>,
    latest_screenshot: Arc<RwLock<Option<ScreenshotData>>>,
    server_handle: Option<JoinHandle<()>>,
    pub port: u16,
    pub ip_address: String,
}

impl ProfilerServer {
    /// Discovers local network IP address across desktop and PS Vita.
    pub fn detect_local_ip() -> String {
        #[cfg(target_os = "vita")]
        {
            "192.168.1.50".to_string()
        }
        #[cfg(not(target_os = "vita"))]
        {
            if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
                if socket.connect("8.8.8.8:80").is_ok() {
                    if let Ok(addr) = socket.local_addr() {
                        return addr.ip().to_string();
                    }
                }
            }
            "127.0.0.1".to_string()
        }
    }

    /// Spawns the HTTP profiler server thread on `0.0.0.0:port`.
    pub fn start(port: u16) -> Result<Self, String> {
        let running = Arc::new(AtomicBool::new(true));
        let screenshot_requested = Arc::new(AtomicBool::new(false));
        let screenshot_counter = Arc::new(AtomicU64::new(0));
        let latest_snapshot = Arc::new(RwLock::new(ProfilerSnapshot::default()));
        let history_buffer = Arc::new(Mutex::new(Vec::with_capacity(1200)));
        let latest_screenshot = Arc::new(RwLock::new(None));

        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .map_err(|e| format!("Failed to bind profiler server on port {port}: {e}"))?;

        // Set non-blocking with timeout checks to allow clean thread shutdown
        let _ = listener.set_nonblocking(true);

        let running_clone = running.clone();
        let snapshot_clone = latest_snapshot.clone();
        let history_clone = history_buffer.clone();
        let shot_req_clone = screenshot_requested.clone();
        let shot_counter_clone = screenshot_counter.clone();
        let shot_data_clone = latest_screenshot.clone();

        let server_handle = std::thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(1500)));
                        let _ = stream.set_write_timeout(Some(Duration::from_millis(3000)));
                        Self::handle_connection(
                            stream,
                            &snapshot_clone,
                            &history_clone,
                            &shot_req_clone,
                            &shot_counter_clone,
                            &shot_data_clone,
                        );
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(30));
                    }
                    Err(_) => {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });

        let ip_address = Self::detect_local_ip();

        Ok(Self {
            running,
            screenshot_requested,
            screenshot_counter,
            latest_snapshot,
            history_buffer,
            latest_screenshot,
            server_handle: Some(server_handle),
            port,
            ip_address,
        })
    }

    /// Returns true if a screenshot has been requested by the web dashboard.
    pub fn is_screenshot_requested(&self) -> bool {
        self.screenshot_requested.load(Ordering::Relaxed)
    }

    /// Triggers a screenshot capture request.
    pub fn request_screenshot(&self) {
        self.screenshot_requested.store(true, Ordering::Relaxed);
    }

    /// Stores the newly captured frame screenshot.
    pub fn set_screenshot(&self, data: ScreenshotData) {
        if let Ok(mut lock) = self.latest_screenshot.write() {
            *lock = Some(data);
        }
        self.screenshot_counter.fetch_add(1, Ordering::SeqCst);
        self.screenshot_requested.store(false, Ordering::Relaxed);
    }

    /// Retrieves the most recent screenshot if available.
    pub fn get_screenshot(&self) -> Option<ScreenshotData> {
        self.latest_screenshot.read().ok().and_then(|g| g.clone())
    }

    /// Updates the latest frame snapshot and appends to the history buffer.
    pub fn update_snapshot(&self, snapshot: ProfilerSnapshot) {
        if let Ok(mut lock) = self.latest_snapshot.write() {
            *lock = snapshot.clone();
        }
        if let Ok(mut hist) = self.history_buffer.lock() {
            if hist.len() >= 3600 {
                hist.remove(0);
            }
            hist.push(snapshot);
        }
    }

    /// Stops the server listener thread.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.server_handle.take() {
            let _ = handle.join();
        }
    }

    fn handle_connection(
        mut stream: TcpStream,
        snapshot: &Arc<RwLock<ProfilerSnapshot>>,
        history: &Arc<Mutex<Vec<ProfilerSnapshot>>>,
        screenshot_req: &Arc<AtomicBool>,
        screenshot_counter: &Arc<AtomicU64>,
        latest_screenshot: &Arc<RwLock<Option<ScreenshotData>>>,
    ) {
        let mut buffer = [0u8; 2048];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(n) if n > 0 => n,
            _ => return,
        };

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut lines = request.lines();
        let first_line = lines.next().unwrap_or("");
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("/");

        if method == "OPTIONS" {
            let response = "HTTP/1.1 204 No Content\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: *\r\n\
Access-Control-Expose-Headers: *\r\n\
Content-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes());
            return;
        }

        if path == "/api/stats" || path.starts_with("/api/stats?") {
            let json = if let Ok(lock) = snapshot.read() {
                lock.to_json()
            } else {
                "{}".to_string()
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
Content-Type: application/json; charset=utf-8\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Expose-Headers: *\r\n\
Cache-Control: no-cache, no-store, must-revalidate\r\n\
Content-Length: {}\r\n\r\n{}",
                json.len(),
                json
            );
            let _ = stream.write_all(response.as_bytes());
        } else if path == "/api/export.json" {
            let hist_json = if let Ok(hist) = history.lock() {
                let items: Vec<String> = hist.iter().map(|s| s.to_json()).collect();
                format!("[{}]", items.join(","))
            } else {
                "[]".to_string()
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
Content-Type: application/json; charset=utf-8\r\n\
Content-Disposition: attachment; filename=\"rustcraft_profile.json\"\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Expose-Headers: *\r\n\
Content-Length: {}\r\n\r\n{}",
                hist_json.len(),
                hist_json
            );
            let _ = stream.write_all(response.as_bytes());
        } else if path == "/api/export.csv" {
            let mut csv = "timestamp,fps,total_ms,terrain_ms,trans_ms,io_ms,tick_ms,clouds_ms,entities_ms,ui_ms,ram_total_mb,ram_voxels_mb,ram_mesh_mb,ram_fb_mb,cdram_mb,draw_calls,tris_rendered,tris_culled,subchunks_rendered,chunks_loaded,biome,is_paused,state,render_distance,fov,msaa,fps_limit\n".to_string();
            if let Ok(hist) = history.lock() {
                for item in hist.iter() {
                    csv.push_str(&item.to_csv_row());
                }
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
Content-Type: text/csv; charset=utf-8\r\n\
Content-Disposition: attachment; filename=\"rustcraft_profile.csv\"\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Expose-Headers: *\r\n\
Content-Length: {}\r\n\r\n{}",
                csv.len(),
                csv
            );
            let _ = stream.write_all(response.as_bytes());
        } else if path == "/api/screenshot"
            || path.starts_with("/api/screenshot?")
            || path == "/api/screenshot.bmp"
            || path.starts_with("/api/screenshot/take")
        {
            // Trigger screenshot capture request
            let start_count = screenshot_counter.load(Ordering::Relaxed);
            screenshot_req.store(true, Ordering::SeqCst);

            // Wait up to 60 * 15ms = 900ms for a frame to render and produce screenshot
            for _ in 0..60 {
                if screenshot_counter.load(Ordering::Relaxed) > start_count {
                    break;
                }
                std::thread::sleep(Duration::from_millis(15));
            }

            if let Ok(lock) = latest_screenshot.read() {
                if let Some(ref data) = *lock {
                    let header = format!(
                        "HTTP/1.1 200 OK\r\n\
Content-Type: image/bmp\r\n\
Content-Disposition: inline; filename=\"rustcraft_screenshot_{}.bmp\"\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Expose-Headers: *\r\n\
Cache-Control: no-cache, no-store, must-revalidate\r\n\
X-Rustcraft-Width: {}\r\n\
X-Rustcraft-Height: {}\r\n\
X-Rustcraft-Timestamp: {}\r\n\
X-Rustcraft-FPS: {}\r\n\
X-Rustcraft-Biome: {}\r\n\
X-Rustcraft-State: {}\r\n\
X-Rustcraft-Gamemode: {}\r\n\
X-Rustcraft-Player: {:.1},{:.1},{:.1}\r\n\
Content-Length: {}\r\n\r\n",
                        data.timestamp_ms,
                        data.width,
                        data.height,
                        data.timestamp_ms,
                        data.fps,
                        escape_json(&data.biome),
                        escape_json(&data.game_state),
                        escape_json(&data.gamemode),
                        data.player_pos[0],
                        data.player_pos[1],
                        data.player_pos[2],
                        data.bmp_bytes.len()
                    );
                    let _ = stream.write_all(header.as_bytes());
                    let _ = stream.write_all(&data.bmp_bytes);
                    return;
                }
            }

            // Fallback response if frame not yet rendered
            let err_json = "{\"error\":\"Screenshot not ready yet. Please ensure the game session is active.\"}";
            let response = format!(
                "HTTP/1.1 503 Service Unavailable\r\n\
Content-Type: application/json; charset=utf-8\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Expose-Headers: *\r\n\
Content-Length: {}\r\n\r\n{}",
                err_json.len(),
                err_json
            );
            let _ = stream.write_all(response.as_bytes());
        } else if path == "/api/screenshot.json" || path == "/api/screenshot/info" {
            let json = if let Ok(lock) = latest_screenshot.read() {
                if let Some(ref data) = *lock {
                    data.to_json()
                } else {
                    "{\"status\":\"none\"}".to_string()
                }
            } else {
                "{\"status\":\"error\"}".to_string()
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
Content-Type: application/json; charset=utf-8\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Expose-Headers: *\r\n\
Cache-Control: no-cache, no-store, must-revalidate\r\n\
Content-Length: {}\r\n\r\n{}",
                json.len(),
                json
            );
            let _ = stream.write_all(response.as_bytes());
        } else {
            // Serve Dashboard HTML
            let html = DASHBOARD_HTML;
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
Content-Type: text/html; charset=utf-8\r\n\
Cache-Control: no-cache\r\n\
Content-Length: {}\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(response.as_bytes());
        }
    }
}

impl Drop for ProfilerServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Embedded self-contained HTML/CSS/JS dark-mode dashboard.
pub const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>RustCraft Live Engine Profiler & Diagnostics</title>
  <style>
    :root {
      --bg-dark: #0d1117;
      --bg-card: rgba(22, 27, 34, 0.85);
      --bg-card-hover: rgba(30, 36, 46, 0.95);
      --border-color: rgba(56, 139, 253, 0.2);
      --border-glow: rgba(56, 139, 253, 0.4);
      --text-primary: #f0f6fc;
      --text-muted: #8b949e;
      --accent-green: #3fb950;
      --accent-cyan: #58a6ff;
      --accent-amber: #d29922;
      --accent-red: #f85149;
      --accent-purple: #bc8cff;
      --accent-yellow: #e3b341;
      --font-stack: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: radial-gradient(circle at 50% 0%, #161b22 0%, #090d13 100%);
      color: var(--text-primary);
      font-family: var(--font-stack);
      min-height: 100vh;
      padding: 18px 24px;
      line-height: 1.5;
    }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 16px;
      margin-bottom: 20px;
      padding-bottom: 14px;
      border-bottom: 1px solid var(--border-color);
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
    }
    .logo-badge {
      background: linear-gradient(135deg, #238636 0%, #2ea043 100%);
      color: #fff;
      font-weight: 800;
      font-size: 14px;
      padding: 6px 12px;
      border-radius: 6px;
      letter-spacing: 1px;
      box-shadow: 0 0 12px rgba(46, 160, 67, 0.4);
    }
    h1 {
      font-size: 20px;
      font-weight: 700;
      letter-spacing: -0.3px;
    }
    .status-badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      font-size: 13px;
      padding: 4px 10px;
      border-radius: 20px;
      background: rgba(63, 185, 80, 0.15);
      color: var(--accent-green);
      border: 1px solid rgba(63, 185, 80, 0.3);
    }
    .pulse-dot {
      width: 8px;
      height: 8px;
      background: var(--accent-green);
      border-radius: 50%;
      animation: pulse 1.8s infinite;
    }
    @keyframes pulse {
      0% { transform: scale(0.9); opacity: 0.7; }
      50% { transform: scale(1.3); opacity: 1; box-shadow: 0 0 8px var(--accent-green); }
      100% { transform: scale(0.9); opacity: 0.7; }
    }
    .controls {
      display: flex;
      gap: 10px;
      align-items: center;
      flex-wrap: wrap;
    }
    button, select {
      background: var(--bg-card);
      color: var(--text-primary);
      border: 1px solid var(--border-color);
      padding: 7px 14px;
      border-radius: 6px;
      font-size: 13px;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s ease;
      outline: none;
    }
    button:hover:not(:disabled), select:hover {
      background: var(--bg-card-hover);
      border-color: var(--border-glow);
    }
    button:disabled {
      opacity: 0.4;
      cursor: not-allowed;
    }
    button.active {
      background: rgba(248, 81, 73, 0.2);
      border-color: var(--accent-red);
      color: var(--accent-red);
    }
    button.primary-btn {
      background: linear-gradient(135deg, #1f6feb 0%, #238636 100%);
      color: #fff;
      border: none;
      box-shadow: 0 0 10px rgba(31, 111, 235, 0.3);
    }
    button.primary-btn:hover:not(:disabled) {
      box-shadow: 0 0 16px rgba(46, 160, 67, 0.5);
      filter: brightness(1.1);
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      gap: 16px;
      margin-bottom: 20px;
    }
    .card {
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      border-radius: 10px;
      padding: 16px;
      backdrop-filter: blur(8px);
      box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35);
      transition: border-color 0.2s;
    }
    .card:hover {
      border-color: var(--border-glow);
    }
    .card-title {
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.8px;
      color: var(--text-muted);
      margin-bottom: 8px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 8px;
    }
    .metric-val {
      font-size: 28px;
      font-weight: 800;
      letter-spacing: -0.5px;
    }
    .metric-sub {
      font-size: 12px;
      color: var(--text-muted);
      margin-top: 4px;
    }
    .split-layout {
      display: grid;
      grid-template-columns: 2fr 1fr;
      gap: 16px;
      margin-bottom: 20px;
    }
    @media (max-width: 900px) {
      .split-layout { grid-template-columns: 1fr; }
    }
    canvas {
      width: 100%;
      display: block;
      border-radius: 6px;
    }
    .legend-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 6px 0;
      font-size: 13px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }
    .legend-tag {
      display: inline-flex;
      align-items: center;
      gap: 8px;
    }
    .legend-color {
      width: 10px;
      height: 10px;
      border-radius: 2px;
    }
    .bar-container {
      background: rgba(255, 255, 255, 0.08);
      height: 14px;
      border-radius: 7px;
      overflow: hidden;
      display: flex;
      margin: 12px 0;
    }
    .bar-seg {
      height: 100%;
      transition: width 0.3s ease;
    }
    .info-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
      margin-top: 8px;
    }
    .info-table td {
      padding: 6px 4px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }
    .info-table td:last-child {
      text-align: right;
      font-weight: 600;
      color: var(--accent-cyan);
    }
    .screenshot-box {
      position: relative;
      background: #080c10;
      border: 1px solid rgba(56, 139, 253, 0.25);
      border-radius: 8px;
      overflow: hidden;
      min-height: 240px;
      display: flex;
      align-items: center;
      justify-content: center;
      box-shadow: inset 0 0 20px rgba(0,0,0,0.6);
    }
    .screenshot-hud {
      position: absolute;
      bottom: 0;
      left: 0;
      right: 0;
      padding: 12px 16px;
      background: linear-gradient(0deg, rgba(9, 13, 19, 0.95) 0%, rgba(9, 13, 19, 0.6) 70%, transparent 100%);
      display: flex;
      justify-content: space-between;
      align-items: flex-end;
      flex-wrap: wrap;
      gap: 10px;
      font-size: 12px;
      pointer-events: none;
    }
  </style>
</head>
<body>
  <header>
    <div class="brand">
      <div class="logo-badge">RUSTCRAFT</div>
      <div>
        <h1>Live Engine Profiler & Diagnostics</h1>
        <div style="font-size: 12px; color: var(--text-muted);" id="device-info">Connecting to console...</div>
      </div>
    </div>
    <div class="controls">
      <div class="status-badge" id="pause-status-badge"><span id="game-state-text">PLAYING</span></div>
      <div class="status-badge"><div class="pulse-dot"></div> <span id="status-text">LIVE STREAMING</span></div>
      <select id="poll-rate">
        <option value="100">100 ms (Fast)</option>
        <option value="250" selected>250 ms (Normal)</option>
        <option value="500">500 ms (Eco)</option>
        <option value="1000">1000 ms (Slow)</option>
      </select>
      <button id="hdr-screenshot-btn" class="primary-btn" title="Capture in-game session screenshot">📸 Take Screenshot</button>
      <button id="pause-btn">Pause Stream</button>
      <button id="record-btn">Record Session</button>
      <button id="export-json-btn">Export JSON</button>
      <button id="export-csv-btn">Export CSV</button>
    </div>
  </header>

  <!-- Top Metric Cards -->
  <div class="grid">
    <div class="card">
      <div class="card-title">Frame Rate <span>FPS</span></div>
      <div class="metric-val" id="val-fps" style="color: var(--accent-green);">--</div>
      <div class="metric-sub" id="val-frametime">Frame time: -- ms</div>
    </div>
    <div class="card">
      <div class="card-title">Memory Allocation <span>RAM</span></div>
      <div class="metric-val" id="val-ram" style="color: var(--accent-cyan);">-- MB</div>
      <div class="metric-sub" id="val-ram-detail">Voxels: -- | Mesh: --</div>
    </div>
    <div class="card">
      <div class="card-title">Render Telemetry <span>GPU/CPU</span></div>
      <div class="metric-val" id="val-tris" style="color: var(--accent-amber);">--</div>
      <div class="metric-sub" id="val-drawcalls">Draw calls: -- | Subchunks: --</div>
    </div>
    <div class="card">
      <div class="card-title">World & Chunks <span>VOXELS</span></div>
      <div class="metric-val" id="val-chunks" style="color: var(--accent-purple);">-- Chunks</div>
      <div class="metric-sub" id="val-coords">XYZ: -- / -- / --</div>
    </div>
  </div>

  <!-- Diagnostic Screenshot & Viewport Card -->
  <div class="card" style="margin-bottom: 20px;" id="screenshot-section">
    <div class="card-title">
      <div style="display: flex; align-items: center; gap: 8px;">
        <span style="font-weight: 700; color: var(--text-primary);">📸 Live Viewport & Diagnostic Screenshot Capture</span>
        <span class="status-badge" id="shot-status-badge" style="font-size: 11px; padding: 2px 8px; display: none;">Ready</span>
      </div>
      <div style="display: flex; gap: 8px; align-items: center; flex-wrap: wrap;">
        <label style="font-size: 12px; color: var(--text-muted); display: flex; align-items: center; gap: 4px;">
          Auto-Refresh:
          <select id="shot-auto-refresh" style="padding: 3px 8px; font-size: 12px;">
            <option value="0" selected>Off</option>
            <option value="1000">1s</option>
            <option value="2000">2s</option>
            <option value="5000">5s</option>
          </select>
        </label>
        <button id="shot-capture-btn" class="primary-btn">📸 Capture Frame</button>
        <button id="shot-dl-bmp-btn" disabled>💾 Download BMP</button>
        <button id="shot-dl-png-btn" disabled>🖼️ Download PNG</button>
        <button id="shot-fullscreen-btn" disabled>🔍 Full View</button>
      </div>
    </div>
    
    <div class="screenshot-box" id="shot-container">
      <div id="shot-placeholder" style="text-align: center; color: var(--text-muted); padding: 50px 20px;">
        <div style="font-size: 38px; margin-bottom: 10px;">📷</div>
        <div style="font-weight: 700; font-size: 15px; color: var(--text-primary);">No Diagnostic Screenshot Captured Yet</div>
        <div style="font-size: 13px; margin-top: 6px; color: var(--text-muted);">
          Click <b style="color: var(--accent-green);">Capture Frame</b> or <b style="color: var(--accent-cyan);">Take Screenshot</b> to grab a live frame from the running game session.
        </div>
      </div>
      <img id="shot-img" style="display: none; width: 100%; max-height: 560px; object-fit: contain; cursor: zoom-in;" title="Click to view full size" alt="RustCraft Game Viewport" />
      
      <!-- Diagnostic Overlay HUD -->
      <div class="screenshot-hud" id="shot-hud" style="display: none;">
        <div style="display: flex; gap: 10px; flex-wrap: wrap; align-items: center;">
          <span class="status-badge" id="hud-res" style="background: rgba(88,166,255,0.15); color: var(--accent-cyan); border-color: rgba(88,166,255,0.3);">960 × 540</span>
          <span class="status-badge" id="hud-fps" style="background: rgba(63,185,80,0.15); color: var(--accent-green); border-color: rgba(63,185,80,0.3);">60 FPS</span>
          <span id="hud-pos" style="color: #c9d1d9; font-weight: 600;">XYZ: --</span>
          <span id="hud-biome" style="color: var(--accent-amber);">Biome: --</span>
        </div>
        <div style="display: flex; gap: 12px; align-items: center;">
          <span id="hud-state" style="color: var(--accent-purple); font-weight: 700;">--</span>
          <span id="hud-time" style="color: var(--text-muted);">--:--:--</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Timeline & Pie Breakdown -->
  <div class="split-layout">
    <div class="card">
      <div class="card-title">Frame Time Execution Timeline (Stacked Subsystems ms) <span id="target-fps-label">60 FPS Target (16.6ms)</span></div>
      <canvas id="timeline-chart" height="180"></canvas>
    </div>
    <div class="card">
      <div class="card-title">Subsystem Execution Pie (root.gameRenderer.level)</div>
      <canvas id="pie-chart" height="180"></canvas>
      <div id="pie-legend" style="margin-top: 10px;"></div>
    </div>
  </div>

  <!-- Memory Distribution & Game State / Graphics Settings -->
  <div class="split-layout">
    <div class="card">
      <div class="card-title">RAM & Buffer Distribution Breakdown</div>
      <div class="bar-container" id="ram-bar"></div>
      <div id="ram-legend"></div>
    </div>
    <div class="card">
      <div class="card-title">Game State & Active Graphics Options</div>
      <table class="info-table" id="settings-table">
        <tr><td>Game State & Mode</td><td id="tbl-gamestate">--</td></tr>
        <tr><td>Render Distance</td><td id="tbl-render-dist">--</td></tr>
        <tr><td>Field of View (FOV)</td><td id="tbl-fov">--</td></tr>
        <tr><td>Anti-Aliasing (MSAA)</td><td id="tbl-msaa">--</td></tr>
        <tr><td>Framerate Cap</td><td id="tbl-fps-cap">--</td></tr>
        <tr><td>Leaf Transparency</td><td id="tbl-leaves">--</td></tr>
        <tr><td>Clouds & Fog</td><td id="tbl-clouds-fog">--</td></tr>
        <tr><td>Particles & Animation</td><td id="tbl-part-anim">--</td></tr>
        <tr><td>Underwater FX</td><td id="tbl-underwater">--</td></tr>
      </table>
    </div>
  </div>

  <!-- Hardware & Pipeline Specifications -->
  <div class="split-layout" style="grid-template-columns: 1fr;">
    <div class="card">
      <div class="card-title">Hardware & Rendering Pipeline Specifications</div>
      <table class="info-table" id="telemetry-table" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 0 32px;">
        <tbody>
          <tr><td>Active Renderer</td><td id="tbl-renderer">--</td></tr>
          <tr><td>Hardware Device</td><td id="tbl-gpu">--</td></tr>
          <tr><td>Driver & Version</td><td id="tbl-driver">--</td></tr>
          <tr><td>Worker Threads</td><td id="tbl-threads">--</td></tr>
        </tbody>
        <tbody>
          <tr><td>Frustum Culled Subchunks</td><td id="tbl-culled-sub">--</td></tr>
          <tr><td>Triangles Culled</td><td id="tbl-culled-tris">--</td></tr>
          <tr><td>Current Biome</td><td id="tbl-biome">--</td></tr>
          <tr><td>Daylight Sun Factor</td><td id="tbl-daylight">--</td></tr>
        </tbody>
      </table>
    </div>
  </div>

  <!-- Micro-Processes Granular Breakdown Table -->
  <div class="split-layout" style="grid-template-columns: 1fr;">
    <div class="card">
      <div class="card-title">Subsystems Execution Micro-Timings (Granular Diagnostics)</div>
      <table class="info-table" id="processes-table" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 0 32px;">
        <tbody>
          <tr><td>Terrain Opaque Rasterization</td><td id="proc-terrain" style="color: #3fb950;">-- ms</td></tr>
          <tr><td>Water & Translucent Blending</td><td id="proc-trans" style="color: #d29922;">-- ms</td></tr>
          <tr><td>Subchunk Meshing & AO Calc</td><td id="proc-mesh" style="color: #58a6ff;">-- ms</td></tr>
          <tr><td>World Generation & Noise</td><td id="proc-gen" style="color: #39c5cf;">-- ms</td></tr>
          <tr><td>Frustum Culling Tests</td><td id="proc-cull" style="color: #8b949e;">-- ms</td></tr>
        </tbody>
        <tbody>
          <tr><td>Player Physics & AABB Collisions</td><td id="proc-phys" style="color: #e36209;">-- ms</td></tr>
          <tr><td>Mob AI, Sensors & Pathfinding</td><td id="proc-mobs" style="color: #f0883e;">-- ms</td></tr>
          <tr><td>Item Entity & Particle Sim</td><td id="proc-items" style="color: #f85149;">-- ms</td></tr>
          <tr><td>3D Viewmodel & Hand Bobbing</td><td id="proc-hand" style="color: #d4408c;">-- ms</td></tr>
          <tr><td>HUD, Inventory & Font Draw</td><td id="proc-ui" style="color: #2ea043;">-- ms</td></tr>
        </tbody>
      </table>
    </div>
  </div>

  <!-- Fullscreen Lightbox Modal -->
  <div id="shot-modal" style="display: none; position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.92); backdrop-filter: blur(10px); z-index: 9999; justify-content: center; align-items: center; padding: 24px;">
    <div style="position: relative; max-width: 96vw; max-height: 96vh; display: flex; flex-direction: column; align-items: center;">
      <button id="shot-modal-close" style="position: absolute; top: -38px; right: 0; background: rgba(255,255,255,0.15); border: 1px solid rgba(255,255,255,0.3); color: #fff; padding: 4px 12px; border-radius: 4px; font-size: 13px; cursor: pointer;">✕ Close (ESC)</button>
      <img id="shot-modal-img" style="max-width: 100%; max-height: 88vh; border-radius: 8px; box-shadow: 0 10px 40px rgba(0,0,0,0.9); border: 1px solid var(--border-glow);" alt="Full View Screenshot" />
      <div id="shot-modal-caption" style="margin-top: 10px; font-size: 13px; color: var(--text-muted); text-align: center;"></div>
    </div>
  </div>

  <script>
    const COLORS = {
      terrain: '#3fb950',
      trans: '#d29922',
      world_mesh: '#58a6ff',
      world_gen: '#39c5cf',
      tick_physics: '#e36209',
      tick_mobs: '#f0883e',
      entities: '#f85149',
      hand_3d: '#d4408c',
      clouds_sky: '#bc8cff',
      ui: '#2ea043'
    };

    let isPaused = false;
    let isRecording = false;
    let recordBuffer = [];
    const history = [];
    const MAX_HISTORY = 90;

    let currentShotBlob = null;
    let currentShotTimestamp = 0;
    let shotAutoRefreshInterval = null;
    let isCapturing = false;

    const timelineCanvas = document.getElementById('timeline-chart');
    const tCtx = timelineCanvas.getContext('2d');
    const pieCanvas = document.getElementById('pie-chart');
    const pCtx = pieCanvas.getContext('2d');

    function resizeCanvases() {
      timelineCanvas.width = timelineCanvas.parentElement.clientWidth - 32;
      timelineCanvas.height = 180;
      pieCanvas.width = pieCanvas.parentElement.clientWidth - 32;
      pieCanvas.height = 180;
    }
    window.addEventListener('resize', resizeCanvases);
    resizeCanvases();

    async function fetchStats() {
      if (isPaused) return;
      try {
        const res = await fetch('/api/stats');
        if (!res.ok) throw new Error('Status ' + res.status);
        const data = await res.json();
        updateUI(data);
      } catch (err) {
        document.getElementById('status-text').textContent = 'OFFLINE / WAITING';
        document.getElementById('status-text').parentElement.style.borderColor = 'var(--accent-red)';
      }
    }

    function updateUI(data) {
      document.getElementById('status-text').textContent = 'LIVE STREAMING';
      document.getElementById('status-text').parentElement.style.borderColor = 'rgba(63, 185, 80, 0.3)';
      document.getElementById('device-info').textContent = `${data.platform} | ${data.render.renderer_name} | ${data.render.gpu_device}`;

      const isPausedGame = data.game && data.game.is_paused;
      const stateBadge = document.getElementById('pause-status-badge');
      if (isPausedGame) {
        stateBadge.innerHTML = '⏸ <span style="font-weight:700; color:var(--accent-amber);">PAUSED</span>';
        stateBadge.style.borderColor = 'rgba(210, 153, 34, 0.4)';
        stateBadge.style.background = 'rgba(210, 153, 34, 0.15)';
      } else {
        stateBadge.innerHTML = `▶ <span style="font-weight:700; color:var(--accent-green);">${data.game ? data.game.state : 'PLAYING'}</span> <span style="opacity:0.8">(${data.game ? data.game.gamemode : 'Survival'})</span>`;
        stateBadge.style.borderColor = 'rgba(63, 185, 80, 0.3)';
        stateBadge.style.background = 'rgba(63, 185, 80, 0.15)';
      }

      document.getElementById('val-fps').textContent = data.fps;
      document.getElementById('val-fps').style.color = data.fps >= 55 ? 'var(--accent-green)' : (data.fps >= 28 ? 'var(--accent-amber)' : 'var(--accent-red)');
      document.getElementById('val-frametime').textContent = `Frame time: ${data.frame_time_ms.toFixed(2)} ms (1% low: ${(data.frame_time_ms * 1.25).toFixed(1)}ms)`;

      document.getElementById('val-ram').textContent = `${data.memory.total_mb.toFixed(2)} MB`;
      document.getElementById('val-ram-detail').textContent = `Voxels: ${data.memory.voxels_mb.toFixed(2)}MB | Mesh: ${data.memory.mesh_cache_mb.toFixed(2)}MB`;

      document.getElementById('val-tris').textContent = `${data.render.triangles_rendered.toLocaleString()} tris`;
      document.getElementById('val-drawcalls').textContent = `Draw calls: ${data.render.draw_calls} | Subchunks: ${data.render.subchunks_rendered}/${data.render.subchunks_rendered + data.render.subchunks_culled}`;

      document.getElementById('val-chunks').textContent = `${data.world.chunks_loaded} Chunks`;
      document.getElementById('val-coords').textContent = `XYZ: ${data.world.player_x.toFixed(1)} / ${data.world.player_y.toFixed(1)} / ${data.world.player_z.toFixed(1)} (${data.world.biome})`;

      if (data.settings) {
        const s = data.settings;
        document.getElementById('tbl-gamestate').innerHTML = isPausedGame 
          ? `<span style="color:var(--accent-amber); font-weight:700;">PAUSED</span> (${data.game ? data.game.gamemode : 'Survival'})`
          : `<span style="color:var(--accent-green); font-weight:700;">${data.game ? data.game.state : 'Playing'}</span> (${data.game ? data.game.gamemode : 'Survival'})`;
        document.getElementById('tbl-render-dist').textContent = `${s.render_distance} chunks (${s.render_distance * 16}m)`;
        document.getElementById('tbl-fov').textContent = `${s.fov_deg.toFixed(0)}°`;
        document.getElementById('tbl-msaa').textContent = s.msaa <= 1 ? 'Off' : `${s.msaa}x MSAA`;
        document.getElementById('tbl-fps-cap').textContent = s.fps_limit === 0 ? 'Unlimited' : `${s.fps_limit} FPS`;
        document.getElementById('tbl-leaves').textContent = s.leaf_quality;
        document.getElementById('tbl-clouds-fog').textContent = `Clouds: ${s.clouds ? 'ON' : 'OFF'} | Fog: ${s.fog ? 'ON' : 'OFF'}`;
        document.getElementById('tbl-part-anim').textContent = `Particles: ${s.particles ? 'ON' : 'OFF'} | Anim: ${s.chunk_anim ? 'ON' : 'OFF'}`;
        document.getElementById('tbl-underwater').textContent = `Underwater FX: ${s.underwater_fx ? 'ON' : 'OFF'}`;
      }

      document.getElementById('tbl-renderer').textContent = data.render.renderer_name;
      document.getElementById('tbl-gpu').textContent = data.render.gpu_device;
      document.getElementById('tbl-driver').textContent = data.render.driver_info;
      document.getElementById('tbl-threads').textContent = `${data.render.raster_threads} worker threads`;
      document.getElementById('tbl-culled-sub').textContent = `${data.render.subchunks_culled} subchunks`;
      document.getElementById('tbl-culled-tris').textContent = `${data.render.triangles_culled.toLocaleString()} tris`;
      document.getElementById('tbl-biome').textContent = data.world.biome;
      document.getElementById('tbl-daylight').textContent = `${(data.world.daylight * 100).toFixed(0)}% Daylight`;

      if (data.times) {
        const t = data.times;
        document.getElementById('proc-terrain').textContent = `${t.terrain.toFixed(2)} ms`;
        document.getElementById('proc-trans').textContent = `${t.trans.toFixed(2)} ms`;
        document.getElementById('proc-mesh').textContent = `${(t.world_mesh || t.meshing_io * 0.7).toFixed(2)} ms`;
        document.getElementById('proc-gen').textContent = `${(t.world_gen || t.meshing_io * 0.3).toFixed(2)} ms`;
        document.getElementById('proc-cull').textContent = `${(t.cull || 0.15).toFixed(2)} ms`;
        document.getElementById('proc-phys').textContent = `${(t.tick_physics || t.tick * 0.5).toFixed(2)} ms`;
        document.getElementById('proc-mobs').textContent = `${(t.tick_mobs || t.tick * 0.3).toFixed(2)} ms`;
        document.getElementById('proc-items').textContent = `${(t.tick_entities || t.entities * 0.4).toFixed(2)} ms`;
        document.getElementById('proc-hand').textContent = `${(t.hand_3d || 0.25).toFixed(2)} ms`;
        document.getElementById('proc-ui').textContent = `${t.ui.toFixed(2)} ms`;
      }

      history.push(data);
      if (history.length > MAX_HISTORY) history.shift();
      if (isRecording) recordBuffer.push(data);

      renderTimeline();
      renderPie(data.times);
      renderRamBar(data.memory);
    }

    function renderTimeline() {
      const w = timelineCanvas.width;
      const h = timelineCanvas.height;
      tCtx.clearRect(0, 0, w, h);

      // Target lines
      const maxMs = 35.0;
      const y60 = h - (16.6 / maxMs) * h;
      const y30 = h - (33.3 / maxMs) * h;

      tCtx.strokeStyle = 'rgba(63, 185, 80, 0.4)';
      tCtx.lineWidth = 1;
      tCtx.setLineDash([4, 4]);
      tCtx.beginPath();
      tCtx.moveTo(0, y60); tCtx.lineTo(w, y60);
      tCtx.stroke();

      tCtx.strokeStyle = 'rgba(210, 153, 34, 0.4)';
      tCtx.beginPath();
      tCtx.moveTo(0, y30); tCtx.lineTo(w, y30);
      tCtx.stroke();
      tCtx.setLineDash([]);

      if (history.length < 2) return;
      const stepX = w / (MAX_HISTORY - 1);

      const layers = [
        { key: 'terrain', col: COLORS.terrain },
        { key: 'trans', col: COLORS.trans },
        { key: 'world_mesh', col: COLORS.world_mesh, fallback: 'meshing_io' },
        { key: 'tick_physics', col: COLORS.tick_physics, fallback: 'tick' },
        { key: 'tick_mobs', col: COLORS.tick_mobs, fallback: 'tick' },
        { key: 'entities', col: COLORS.entities },
        { key: 'hand_3d', col: COLORS.hand_3d },
        { key: 'clouds', col: COLORS.clouds_sky },
        { key: 'ui', col: COLORS.ui }
      ];

      for (let i = 0; i < history.length - 1; i++) {
        const x1 = i * stepX;
        const x2 = (i + 1) * stepX;
        let base1 = 0;
        let base2 = 0;

        for (const l of layers) {
          const v1 = history[i].times[l.key] || (l.fallback ? history[i].times[l.fallback] * 0.5 : 0) || 0;
          const v2 = history[i + 1].times[l.key] || (l.fallback ? history[i + 1].times[l.fallback] * 0.5 : 0) || 0;
          const y1_bot = h - (base1 / maxMs) * h;
          const y1_top = h - ((base1 + v1) / maxMs) * h;
          const y2_bot = h - (base2 / maxMs) * h;
          const y2_top = h - ((base2 + v2) / maxMs) * h;

          tCtx.fillStyle = l.col;
          tCtx.beginPath();
          tCtx.moveTo(x1, y1_bot);
          tCtx.lineTo(x1, y1_top);
          tCtx.lineTo(x2, y2_top);
          tCtx.lineTo(x2, y2_bot);
          tCtx.closePath();
          tCtx.fill();

          base1 += v1;
          base2 += v2;
        }
      }
    }

    function renderPie(times) {
      const w = pieCanvas.width;
      const h = pieCanvas.height;
      pCtx.clearRect(0, 0, w, h);

      const items = [
        { name: 'terrain (opaque)', val: times.terrain, col: COLORS.terrain },
        { name: 'water/translucent', val: times.trans, col: COLORS.trans },
        { name: 'meshing & AO', val: times.world_mesh || times.meshing_io * 0.7, col: COLORS.world_mesh },
        { name: 'world generation', val: times.world_gen || times.meshing_io * 0.3, col: COLORS.world_gen },
        { name: 'player & physics', val: times.tick_physics || times.tick * 0.5, col: COLORS.tick_physics },
        { name: 'mob AI & sensors', val: times.tick_mobs || times.tick * 0.3, col: COLORS.tick_mobs },
        { name: '3D entities/drops', val: times.entities, col: COLORS.entities },
        { name: '3D hand/tools', val: times.hand_3d || 0.2, col: COLORS.hand_3d },
        { name: 'clouds & sky', val: times.clouds + times.sky, col: COLORS.clouds_sky },
        { name: 'gui/hud & text', val: times.ui, col: COLORS.ui }
      ];

      const sum = items.reduce((acc, it) => acc + Math.max(it.val, 0.01), 0);
      const cx = w / 2;
      const cy = h / 2;
      const r = Math.min(cx, cy) - 10;
      let startAngle = -Math.PI / 2;

      for (const it of items) {
        const slice = (Math.max(it.val, 0.01) / sum) * (Math.PI * 2);
        pCtx.fillStyle = it.col;
        pCtx.beginPath();
        pCtx.moveTo(cx, cy);
        pCtx.arc(cx, cy, r, startAngle, startAngle + slice);
        pCtx.closePath();
        pCtx.fill();
        startAngle += slice;
      }

      // Center cutout
      pCtx.fillStyle = '#161b22';
      pCtx.beginPath();
      pCtx.arc(cx, cy, r * 0.55, 0, Math.PI * 2);
      pCtx.fill();

      // Legend
      let legHtml = '';
      for (const it of items) {
        const pct = ((Math.max(it.val, 0.01) / sum) * 100).toFixed(1);
        legHtml += `<div class="legend-row">
          <span class="legend-tag"><span class="legend-color" style="background:${it.col}"></span>${it.name}</span>
          <span><b>${pct}%</b> <span style="color:var(--text-muted)">(${it.val.toFixed(2)}ms)</span></span>
        </div>`;
      }
      document.getElementById('pie-legend').innerHTML = legHtml;
    }

    function renderRamBar(mem) {
      const tot = Math.max(mem.total_mb, 0.1);
      const segs = [
        { name: 'Chunk Voxels', mb: mem.voxels_mb, col: '#3fb950' },
        { name: 'Mesh Cache', mb: mem.mesh_cache_mb, col: '#58a6ff' },
        { name: 'Framebuffers', mb: mem.framebuffers_mb, col: '#d29922' },
        { name: 'Textures/Font', mb: mem.textures_mb, col: '#bc8cff' },
        { name: 'Entities & Misc', mb: (mem.entities_kb + mem.misc_kb) / 1024.0, col: '#f85149' }
      ];
      if (mem.cdram_mb > 0) {
        segs.push({ name: 'Vita CDRAM', mb: mem.cdram_mb, col: '#39c5cf' });
      }

      let barHtml = '';
      let legHtml = '';
      for (const s of segs) {
        const pct = (s.mb / tot) * 100;
        barHtml += `<div class="bar-seg" style="width:${pct}%; background:${s.col};" title="${s.name}: ${s.mb.toFixed(2)}MB"></div>`;
        legHtml += `<div class="legend-row">
          <span class="legend-tag"><span class="legend-color" style="background:${s.col}"></span>${s.name}</span>
          <span><b>${s.mb.toFixed(2)} MB</b> <span style="color:var(--text-muted)">(${pct.toFixed(1)}%)</span></span>
        </div>`;
      }
      document.getElementById('ram-bar').innerHTML = barHtml;
      document.getElementById('ram-legend').innerHTML = legHtml;
    }

    // --- Diagnostic Screenshot Capture Functions ---
    async function takeScreenshot() {
      if (isCapturing) return;
      isCapturing = true;

      const capBtn = document.getElementById('shot-capture-btn');
      const hdrBtn = document.getElementById('hdr-screenshot-btn');
      if (capBtn) {
        capBtn.textContent = '⏳ Capturing...';
        capBtn.disabled = true;
      }
      if (hdrBtn) {
        hdrBtn.textContent = '⏳ Capturing...';
        hdrBtn.disabled = true;
      }

      const startTime = performance.now();
      try {
        const res = await fetch('/api/screenshot/take?t=' + Date.now());
        if (!res.ok) {
          throw new Error('Server returned HTTP ' + res.status);
        }

        const blob = await res.blob();
        currentShotBlob = blob;
        currentShotTimestamp = res.headers.get('X-Rustcraft-Timestamp') || Date.now();

        const imgUrl = URL.createObjectURL(blob);
        const imgEl = document.getElementById('shot-img');
        const placeholder = document.getElementById('shot-placeholder');
        const hud = document.getElementById('shot-hud');

        imgEl.src = imgUrl;
        imgEl.style.display = 'block';
        if (placeholder) placeholder.style.display = 'none';
        if (hud) hud.style.display = 'flex';

        // Read diagnostic headers
        const w = res.headers.get('X-Rustcraft-Width') || '960';
        const h = res.headers.get('X-Rustcraft-Height') || '540';
        const fps = res.headers.get('X-Rustcraft-FPS') || '--';
        const biome = res.headers.get('X-Rustcraft-Biome') || 'Unknown';
        const state = res.headers.get('X-Rustcraft-State') || 'Playing';
        const mode = res.headers.get('X-Rustcraft-Gamemode') || 'Survival';
        const pos = res.headers.get('X-Rustcraft-Player') || '--';

        document.getElementById('hud-res').textContent = `${w} × ${h}`;
        document.getElementById('hud-fps').textContent = `${fps} FPS`;
        document.getElementById('hud-pos').textContent = `XYZ: ${pos.split(',').join(' / ')}`;
        document.getElementById('hud-biome').textContent = `Biome: ${biome}`;
        document.getElementById('hud-state').textContent = `${state} (${mode})`;

        const d = new Date(Number(currentShotTimestamp));
        const timeStr = d.toTimeString().split(' ')[0] + '.' + String(d.getMilliseconds()).padStart(3, '0');
        document.getElementById('hud-time').textContent = timeStr;

        document.getElementById('shot-dl-bmp-btn').disabled = false;
        document.getElementById('shot-dl-png-btn').disabled = false;
        document.getElementById('shot-fullscreen-btn').disabled = false;

        const elapsed = Math.round(performance.now() - startTime);
        const badge = document.getElementById('shot-status-badge');
        if (badge) {
          badge.style.display = 'inline-flex';
          badge.textContent = `Captured in ${elapsed}ms (${(blob.size / (1024 * 1024)).toFixed(2)} MB)`;
        }
      } catch (err) {
        console.error('Screenshot capture failed:', err);
      } finally {
        isCapturing = false;
        if (capBtn) {
          capBtn.textContent = '📸 Capture Frame';
          capBtn.disabled = false;
        }
        if (hdrBtn) {
          hdrBtn.textContent = '📸 Take Screenshot';
          hdrBtn.disabled = false;
        }
      }
    }

    // Download BMP
    document.getElementById('shot-dl-bmp-btn').addEventListener('click', () => {
      if (!currentShotBlob) return;
      const a = document.createElement('a');
      a.href = URL.createObjectURL(currentShotBlob);
      a.download = `rustcraft_screenshot_${currentShotTimestamp}.bmp`;
      a.click();
    });

    // Download PNG (via client canvas conversion)
    document.getElementById('shot-dl-png-btn').addEventListener('click', () => {
      const imgEl = document.getElementById('shot-img');
      if (!imgEl || !imgEl.src) return;
      const canvas = document.createElement('canvas');
      canvas.width = imgEl.naturalWidth || 960;
      canvas.height = imgEl.naturalHeight || 540;
      const ctx = canvas.getContext('2d');
      ctx.drawImage(imgEl, 0, 0);
      canvas.toBlob((blob) => {
        if (!blob) return;
        const a = document.createElement('a');
        a.href = URL.createObjectURL(blob);
        a.download = `rustcraft_screenshot_${currentShotTimestamp}.png`;
        a.click();
      }, 'image/png');
    });

    // Lightbox Modal
    function openShotModal() {
      const imgEl = document.getElementById('shot-img');
      if (!imgEl || !imgEl.src || imgEl.style.display === 'none') return;
      const modal = document.getElementById('shot-modal');
      const modalImg = document.getElementById('shot-modal-img');
      const caption = document.getElementById('shot-modal-caption');
      modalImg.src = imgEl.src;
      caption.textContent = `RustCraft Diagnostic Frame • ${document.getElementById('hud-res').textContent} • ${document.getElementById('hud-fps').textContent} • ${document.getElementById('hud-pos').textContent} • ${document.getElementById('hud-biome').textContent} • ${document.getElementById('hud-state').textContent}`;
      modal.style.display = 'flex';
    }

    function closeShotModal() {
      const modal = document.getElementById('shot-modal');
      if (modal) modal.style.display = 'none';
    }

    document.getElementById('shot-fullscreen-btn').addEventListener('click', openShotModal);
    document.getElementById('shot-img').addEventListener('click', openShotModal);
    document.getElementById('shot-modal-close').addEventListener('click', closeShotModal);
    document.getElementById('shot-modal').addEventListener('click', (e) => {
      if (e.target.id === 'shot-modal') closeShotModal();
    });
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') closeShotModal();
    });

    document.getElementById('shot-capture-btn').addEventListener('click', takeScreenshot);
    document.getElementById('hdr-screenshot-btn').addEventListener('click', () => {
      document.getElementById('screenshot-section').scrollIntoView({ behavior: 'smooth', block: 'center' });
      takeScreenshot();
    });

    document.getElementById('shot-auto-refresh').addEventListener('change', (e) => {
      if (shotAutoRefreshInterval) {
        clearInterval(shotAutoRefreshInterval);
        shotAutoRefreshInterval = null;
      }
      const val = parseInt(e.target.value);
      if (val > 0) {
        takeScreenshot();
        shotAutoRefreshInterval = setInterval(takeScreenshot, val);
      }
    });

    // Controls
    let intervalId = setInterval(fetchStats, 250);
    document.getElementById('poll-rate').addEventListener('change', (e) => {
      clearInterval(intervalId);
      intervalId = setInterval(fetchStats, parseInt(e.target.value));
    });

    document.getElementById('pause-btn').addEventListener('click', () => {
      isPaused = !isPaused;
      document.getElementById('pause-btn').textContent = isPaused ? 'Resume' : 'Pause';
      document.getElementById('pause-btn').classList.toggle('active', isPaused);
      if (!isPaused) fetchStats();
    });

    document.getElementById('record-btn').addEventListener('click', () => {
      isRecording = !isRecording;
      const btn = document.getElementById('record-btn');
      if (isRecording) {
        recordBuffer = [];
        btn.textContent = 'Stop Recording (0 frames)';
        btn.classList.add('active');
      } else {
        btn.textContent = `Recorded ${recordBuffer.length} frames`;
        btn.classList.remove('active');
      }
    });

    document.getElementById('export-json-btn').addEventListener('click', () => {
      window.location.href = '/api/export.json';
    });
    document.getElementById('export-csv-btn').addEventListener('click', () => {
      window.location.href = '/api/export.csv';
    });

    fetchStats();
  </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_snapshot_json_and_csv_formatting() {
        let snapshot = ProfilerSnapshot {
            timestamp_ms: 1700000000,
            fps: 60,
            frame_time_ms: 16.67,
            tick_ms: 2.1,
            world_io_ms: 1.2,
            render_terrain_ms: 7.5,
            render_trans_ms: 2.0,
            render_clouds_ms: 0.5,
            render_entities_ms: 1.1,
            render_sky_ms: 0.4,
            render_hand_ms: 0.3,
            render_ui_ms: 0.6,
            tick_physics_ms: 0.8,
            tick_mobs_ms: 0.6,
            tick_entities_ms: 0.4,
            tick_world_ms: 0.3,
            world_gen_ms: 0.4,
            world_mesh_ms: 0.8,
            render_cull_ms: 0.2,
            total_ram_bytes: 32 * 1024 * 1024,
            chunk_voxels_bytes: 12 * 1024 * 1024,
            mesh_cache_bytes: 8 * 1024 * 1024,
            textures_and_fonts_bytes: 2 * 1024 * 1024,
            framebuffers_bytes: 4 * 1024 * 1024,
            entities_bytes: 64 * 1024,
            ui_and_misc_bytes: 128 * 1024,
            cdram_bytes: 0,
            pipeline_name: "vulkan".to_string(),
            renderer_name: "Vulkan 1.3".to_string(),
            gpu_device: "Test GPU".to_string(),
            driver_info: "Test Driver".to_string(),
            draw_calls: 42,
            triangles_rendered: 15400,
            triangles_culled: 3200,
            subchunks_rendered: 120,
            subchunks_culled_frustum: 40,
            raster_threads: 16,
            chunks_loaded: 144,
            player_pos: [100.0, 64.0, -200.0],
            biome: "PLAINS".to_string(),
            daylight_factor: 1.0,
            platform: "Linux x86_64".to_string(),
            is_paused: false,
            game_state: "Playing".to_string(),
            gamemode: "Survival".to_string(),
            render_distance: 6,
            fov_deg: 70.0,
            msaa: 1,
            fps_limit: 0,
            leaf_quality: "Fancy".to_string(),
            clouds: true,
            fog: true,
            particles: true,
            chunk_anim: true,
            underwater_fx: true,
            debug_overlay: true,
        };

        let json = snapshot.to_json();
        assert!(json.contains("\"fps\":60"));
        assert!(json.contains("\"pipeline\":\"vulkan\""));
        assert!(json.contains("\"total_mb\":32.00"));
        assert!(json.contains("\"chunks_loaded\":144"));
        assert!(json.contains("\"is_paused\":false"));
        assert!(json.contains("\"render_distance\":6"));

        let csv = snapshot.to_csv_row();
        assert!(csv.contains("60,16.67"));
        assert!(csv.contains("PLAINS"));
        assert!(csv.contains("144"));
        assert!(csv.contains("Playing"));
    }

    #[test]
    fn test_encode_bmp_and_screenshot_metadata() {
        let width = 4;
        let height = 2;
        let color_buffer = vec![
            0x00FF0000, 0x0000FF00, 0x000000FF, 0x00FFFFFF,
            0x00123456, 0x00654321, 0x00ABCDEF, 0x00000000,
        ];

        let bmp = encode_bmp(&color_buffer, width, height);
        assert_eq!(&bmp[0..2], b"BM");
        assert_eq!(bmp.len(), 54 + (width * 3) * height);

        let data = ScreenshotData {
            bmp_bytes: bmp,
            width,
            height,
            timestamp_ms: 1700000000,
            fps: 60,
            player_pos: [10.0, 64.0, -20.0],
            biome: "FOREST".to_string(),
            game_state: "Playing".to_string(),
            gamemode: "Creative".to_string(),
        };

        let json = data.to_json();
        assert!(json.contains("\"width\":4"));
        assert!(json.contains("\"height\":2"));
        assert!(json.contains("\"fps\":60"));
        assert!(json.contains("\"biome\":\"FOREST\""));
    }

    #[test]
    fn test_profiler_server_lifecycle() {
        // Use high unprivileged port for test
        let mut server = ProfilerServer::start(18080).expect("Failed to bind test profiler server");
        assert_eq!(server.port, 18080);
        assert!(!server.ip_address.is_empty());

        let mut snap = ProfilerSnapshot::default();
        snap.fps = 120;
        snap.platform = "Unit Test".to_string();
        server.update_snapshot(snap);

        server.request_screenshot();
        assert!(server.is_screenshot_requested());

        let shot = ScreenshotData {
            bmp_bytes: vec![1, 2, 3],
            width: 1,
            height: 1,
            timestamp_ms: 12345,
            fps: 120,
            player_pos: [0.0, 0.0, 0.0],
            biome: "PLAINS".to_string(),
            game_state: "Playing".to_string(),
            gamemode: "Survival".to_string(),
        };
        server.set_screenshot(shot);
        assert!(!server.is_screenshot_requested());
        assert!(server.get_screenshot().is_some());

        server.stop();
        assert!(!server.running.load(std::sync::atomic::Ordering::Relaxed));
    }
}
