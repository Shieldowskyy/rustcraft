use crate::core::math::{Mat4, Vec3};

#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // rotation around Y axis, radians
    pub pitch: f32, // rotation around X axis, radians
    pub fov_y: f32,
    pub roll: f32,
}

const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Camera {
            position,
            yaw: -std::f32::consts::FRAC_PI_2, // face -Z initially
            pitch: 0.0,
            fov_y: 70.0_f32.to_radians(),
            roll: 0.0,
        }
    }

    pub fn rotate(&mut self, dyaw: f32, dpitch: f32) {
        self.yaw += dyaw;
        self.pitch = (self.pitch + dpitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Full 3D forward direction (used for raycasting / looking).
    pub fn forward(&self) -> Vec3 {
        let cp = self.pitch.cos();
        Vec3::new(self.yaw.cos() * cp, self.pitch.sin(), self.yaw.sin() * cp).normalized()
    }

    /// Forward direction flattened onto the XZ plane (used for WASD movement).
    pub fn forward_flat(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalized()
    }

    pub fn right_flat(&self) -> Vec3 {
        let f = self.forward_flat();
        f.cross(Vec3::new(0.0, 1.0, 0.0)).normalized()
    }

    pub fn right(&self) -> Vec3 {
        let f = self.forward();
        f.cross(Vec3::new(0.0, 1.0, 0.0)).normalized()
    }

    pub fn up(&self) -> Vec3 {
        let r = self.right();
        let f = self.forward();
        r.cross(f).normalized()
    }

    pub fn rolled_up(&self) -> Vec3 {
        let base_up = Vec3::new(0.0, 1.0, 0.0);
        if self.roll.abs() > 0.00001 {
            let right = self.right_flat();
            (base_up * self.roll.cos() + right * self.roll.sin()).normalized()
        } else {
            base_up
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        let target = self.position + self.forward();
        Mat4::look_at(self.position, target, self.rolled_up())
    }

    pub fn view_matrix_with_tilt(&self, shake_offset: Vec3, roll_rad: f32) -> Mat4 {
        let mut cam = self.clone();
        cam.position += shake_offset;
        cam.roll += roll_rad;
        cam.view_matrix()
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective(self.fov_y, aspect, 0.05, 400.0)
    }
}
