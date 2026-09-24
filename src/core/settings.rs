use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameSettings {
    pub render_distance: i32, // 2, 4, 6, 8, 10, 12, 16 (domyślnie: 6)
    pub fancy_leaves: bool,   // domyślnie: true (prześwitujące vs zwarte)
    pub fog: bool,            // domyślnie: true
    pub clouds: bool,         // domyślnie: true
    pub particles: bool,      // domyślnie: true
    pub chunk_anim: bool,     // domyślnie: true
    pub fov_deg: f32,         // domyślnie: 70.0
    pub underwater_fx: bool,  // domyślnie: true
    pub debug_overlay: bool,  // domyślnie: true
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            render_distance: 6,
            fancy_leaves: true,
            fog: true,
            clouds: true,
            particles: true,
            chunk_anim: true,
            fov_deg: 70.0,
            underwater_fx: true,
            debug_overlay: true,
        }
    }
}

impl GameSettings {
    pub fn load() -> Self {
        let path = Self::settings_path();
        if let Ok(bytes) = std::fs::read(&path) {
            if bytes.len() >= 10 && &bytes[0..4] == b"RCST" {
                let render_distance = i32::from_le_bytes(bytes[4..8].try_into().unwrap_or([6, 0, 0, 0])).clamp(2, 16);
                let clouds = bytes[8] != 0;
                let underwater_fx = bytes.get(9).copied().unwrap_or(1) != 0;
                let fov_deg = if bytes.len() >= 14 {
                    f32::from_le_bytes(bytes[10..14].try_into().unwrap_or([0, 0, 140, 66])).clamp(50.0, 110.0)
                } else {
                    70.0
                };
                let fog = bytes.get(14).copied().unwrap_or(1) != 0;
                let chunk_anim = bytes.get(15).copied().unwrap_or(1) != 0;
                let particles = bytes.get(16).copied().unwrap_or(1) != 0;
                let debug_overlay = bytes.get(17).copied().unwrap_or(1) != 0;
                let fancy_leaves = bytes.get(18).copied().unwrap_or(1) != 0;
                return Self {
                    render_distance,
                    fancy_leaves,
                    fog,
                    clouds,
                    particles,
                    chunk_anim,
                    fov_deg,
                    underwater_fx,
                    debug_overlay,
                };
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut bytes = Vec::with_capacity(19);
        bytes.extend_from_slice(b"RCST");
        bytes.extend_from_slice(&self.render_distance.to_le_bytes());
        bytes.push(if self.clouds { 1 } else { 0 });
        bytes.push(if self.underwater_fx { 1 } else { 0 });
        bytes.extend_from_slice(&self.fov_deg.to_le_bytes());
        bytes.push(if self.fog { 1 } else { 0 });
        bytes.push(if self.chunk_anim { 1 } else { 0 });
        bytes.push(if self.particles { 1 } else { 0 });
        bytes.push(if self.debug_overlay { 1 } else { 0 });
        bytes.push(if self.fancy_leaves { 1 } else { 0 });
        std::fs::write(&path, bytes).map_err(|e| e.to_string())
    }

    fn settings_path() -> PathBuf {
        crate::world::storage::settings_file_path()
    }

    pub fn cycle_render_distance(&mut self, forward: bool) {
        const CHOICES: [i32; 7] = [2, 4, 6, 8, 10, 12, 16];
        let idx = CHOICES.iter().position(|&d| d == self.render_distance).unwrap_or(2);
        let next_idx = if forward {
            (idx + 1) % CHOICES.len()
        } else if idx == 0 {
            CHOICES.len() - 1
        } else {
            idx - 1
        };
        self.render_distance = CHOICES[next_idx];
    }

    pub fn cycle_fov(&mut self, forward: bool) {
        const CHOICES: [f32; 6] = [60.0, 70.0, 80.0, 90.0, 100.0, 110.0];
        let idx = CHOICES.iter().position(|&f| (f - self.fov_deg).abs() < 1.0).unwrap_or(1);
        let next_idx = if forward {
            (idx + 1) % CHOICES.len()
        } else if idx == 0 {
            CHOICES.len() - 1
        } else {
            idx - 1
        };
        self.fov_deg = CHOICES[next_idx];
    }
}
