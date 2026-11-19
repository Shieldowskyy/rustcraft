//! Background TCP HTTP listener serving profiler telemetry, exports, and screenshots.

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::snapshot::escape_json;
use super::{ProfilerSnapshot, ScreenshotData, DASHBOARD_HTML};

pub struct ProfilerServer {
    pub(super) running: Arc<AtomicBool>,
    screenshot_requested: Arc<AtomicBool>,
    screenshot_counter: Arc<AtomicU32>,
    latest_snapshot: Arc<RwLock<ProfilerSnapshot>>,
    history_buffer: Arc<Mutex<Vec<ProfilerSnapshot>>>,
    latest_screenshot: Arc<RwLock<Option<ScreenshotData>>>,
    server_handle: Option<JoinHandle<()>>,
    pub port: u16,
    pub ip_address: String,
}

impl ProfilerServer {
    pub fn detect_local_ip() -> String {
        crate::platform::system::detect_local_ip()
    }

    pub fn start(port: u16) -> Result<Self, String> {
        let running = Arc::new(AtomicBool::new(true));
        let screenshot_requested = Arc::new(AtomicBool::new(false));
        let screenshot_counter = Arc::new(AtomicU32::new(0));
        let latest_snapshot = Arc::new(RwLock::new(ProfilerSnapshot::default()));
        let history_buffer = Arc::new(Mutex::new(Vec::with_capacity(1200)));
        let latest_screenshot = Arc::new(RwLock::new(None));

        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .map_err(|e| format!("Failed to bind profiler server on port {port}: {e}"))?;
        let _ = listener.set_nonblocking(true);

        let running_clone = running.clone();
        let snapshot_clone = latest_snapshot.clone();
        let history_clone = history_buffer.clone();
        let shot_req_clone = screenshot_requested.clone();
        let shot_counter_clone = screenshot_counter.clone();
        let shot_data_clone = latest_screenshot.clone();

        let server_handle = std::thread::spawn(move || {
            crate::platform::threading::configure_background_decoder_affinity();
            while running_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        // DLACZEGO: On PS Vita SceNet accepted sockets inherit SO_NONBLOCK from the
                        // non-blocking TcpListener, truncating 1.57 MB BMP transfers with WouldBlock.
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_nodelay(true);
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(2000)));
                        let _ = stream.set_write_timeout(Some(Duration::from_millis(15_000)));
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

    pub fn is_screenshot_requested(&self) -> bool {
        self.screenshot_requested.load(Ordering::Relaxed)
    }

    pub fn request_screenshot(&self) {
        self.screenshot_requested.store(true, Ordering::Relaxed);
    }

    pub fn set_screenshot(&self, data: ScreenshotData) {
        if let Ok(mut lock) = self.latest_screenshot.write() {
            *lock = Some(data);
        }
        self.screenshot_counter.fetch_add(1, Ordering::Release);
        self.screenshot_requested.store(false, Ordering::Release);
    }

    pub fn get_screenshot(&self) -> Option<ScreenshotData> {
        self.latest_screenshot.read().ok().and_then(|g| g.clone())
    }

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
        screenshot_counter: &Arc<AtomicU32>,
        latest_screenshot: &Arc<RwLock<Option<ScreenshotData>>>,
    ) {
        let mut buffer = [0u8; 2048];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(n) if n > 0 => n,
            _ => return,
        };

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut parts = request.lines().next().unwrap_or("").split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("/");

        if method == "OPTIONS" {
            let response = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\nAccess-Control-Expose-Headers: *\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
            let _ = write_all_reliable(&mut stream, response.as_bytes());
            finish_stream(&mut stream);
            return;
        }

        if path == "/api/stats" || path.starts_with("/api/stats?") {
            let json = snapshot.read().map(|lock| lock.to_json()).unwrap_or_else(|_| "{}".to_string());
            write_response(&mut stream, "application/json; charset=utf-8", None, &json);
        } else if path == "/api/export.json" {
            let hist_json = history
                .lock()
                .map(|hist| format!("[{}]", hist.iter().map(|s| s.to_json()).collect::<Vec<_>>().join(",")))
                .unwrap_or_else(|_| "[]".to_string());
            write_response(
                &mut stream,
                "application/json; charset=utf-8",
                Some("attachment; filename=\"rustcraft_profile.json\""),
                &hist_json,
            );
        } else if path == "/api/export.csv" {
            let mut csv = "timestamp,fps,total_ms,terrain_ms,trans_ms,io_ms,tick_ms,clouds_ms,entities_ms,ui_ms,ram_total_mb,ram_voxels_mb,ram_mesh_mb,ram_fb_mb,cdram_mb,draw_calls,tris_rendered,tris_culled,subchunks_rendered,chunks_loaded,biome,is_paused,state,render_distance,fov,msaa,fps_limit\n".to_string();
            if let Ok(hist) = history.lock() {
                for item in hist.iter() {
                    csv.push_str(&item.to_csv_row());
                }
            }
            write_response(
                &mut stream,
                "text/csv; charset=utf-8",
                Some("attachment; filename=\"rustcraft_profile.csv\""),
                &csv,
            );
        } else if path == "/api/screenshot"
            || path.starts_with("/api/screenshot?")
            || path == "/api/screenshot.bmp"
            || path.starts_with("/api/screenshot/take")
        {
            serve_screenshot(&mut stream, screenshot_req, screenshot_counter, latest_screenshot);
        } else if path == "/api/screenshot.json" || path == "/api/screenshot/info" {
            let json = latest_screenshot
                .read()
                .map(|lock| lock.as_ref().map(|d| d.to_json()).unwrap_or_else(|| "{\"status\":\"none\"}".to_string()))
                .unwrap_or_else(|_| "{\"status\":\"error\"}".to_string());
            write_response(&mut stream, "application/json; charset=utf-8", None, &json);
        } else {
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-cache\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
                DASHBOARD_HTML.len()
            );
            if write_all_reliable(&mut stream, header.as_bytes()).is_ok() {
                let _ = write_all_reliable(&mut stream, DASHBOARD_HTML.as_bytes());
            }
            finish_stream(&mut stream);
        }
    }
}

impl Drop for ProfilerServer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn write_all_reliable(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(15);
    for chunk in data.chunks(16 * 1024) {
        let mut rem = chunk;
        while !rem.is_empty() {
            match stream.write(rem) {
                Ok(0) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => rem = &rem[n..],
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "socket write timed out",
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}

fn finish_stream(stream: &mut TcpStream) {
    let _ = stream.flush();
    let _ = stream.shutdown(Shutdown::Write);
}

fn write_response(stream: &mut TcpStream, content_type: &str, disposition: Option<&str>, body: &str) {
    let disp_hdr = disposition
        .map(|d| format!("Content-Disposition: {d}\r\n"))
        .unwrap_or_default();
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\n{disp_hdr}Access-Control-Allow-Origin: *\r\nAccess-Control-Expose-Headers: *\r\nCache-Control: no-cache, no-store, must-revalidate\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    if write_all_reliable(stream, header.as_bytes()).is_ok() {
        let _ = write_all_reliable(stream, body.as_bytes());
    }
    finish_stream(stream);
}

fn serve_screenshot(
    stream: &mut TcpStream,
    screenshot_req: &Arc<AtomicBool>,
    screenshot_counter: &Arc<AtomicU32>,
    latest_screenshot: &Arc<RwLock<Option<ScreenshotData>>>,
) {
    let start_count = screenshot_counter.load(Ordering::Acquire);
    screenshot_req.store(true, Ordering::SeqCst);

    let mut captured_fresh = false;
    for _ in 0..166 {
        if screenshot_counter.load(Ordering::Acquire) > start_count {
            captured_fresh = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }

    if captured_fresh {
        if let Ok(lock) = latest_screenshot.read() {
            if let Some(ref data) = *lock {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/bmp\r\nContent-Disposition: inline; filename=\"rustcraft_screenshot_{}.bmp\"\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Expose-Headers: *\r\nCache-Control: no-cache, no-store, must-revalidate\r\nConnection: close\r\nX-Rustcraft-Width: {}\r\nX-Rustcraft-Height: {}\r\nX-Rustcraft-Timestamp: {}\r\nX-Rustcraft-FPS: {}\r\nX-Rustcraft-Biome: {}\r\nX-Rustcraft-State: {}\r\nX-Rustcraft-Gamemode: {}\r\nX-Rustcraft-Player: {:.1},{:.1},{:.1}\r\nContent-Length: {}\r\n\r\n",
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
                if write_all_reliable(stream, header.as_bytes()).is_ok() {
                    let _ = write_all_reliable(stream, &data.bmp_bytes);
                }
                finish_stream(stream);
                return;
            }
        }
    }

    let err_json = "{\"error\":\"Screenshot capture timed out. Ensure a renderable frame is running.\"}";
    let response = format!(
        "HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Expose-Headers: *\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        err_json.len(),
        err_json
    );
    let _ = write_all_reliable(stream, response.as_bytes());
    finish_stream(stream);
}

