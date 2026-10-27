// Minimal math library: vectors and a 4x4 matrix, written from scratch so
// the project has zero rendering-related dependencies.

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    #[inline(always)]
    pub const fn zero() -> Self {
        Vec3::new(0.0, 0.0, 0.0)
    }

    #[inline(always)]
    pub fn length(&self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline(always)]
    pub fn length_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline(always)]
    pub fn normalized(&self) -> Vec3 {
        let len = self.length();
        if len < 1e-8 {
            Vec3::zero()
        } else {
            Vec3::new(self.x / len, self.y / len, self.z / len)
        }
    }

    #[inline(always)]
    pub fn dot(&self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    #[inline(always)]
    pub fn cross(&self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    #[inline(always)]
    pub fn floor_to_ivec(&self) -> IVec3 {
        IVec3::new(
            self.x.floor() as i32,
            self.y.floor() as i32,
            self.z.floor() as i32,
        )
    }

    #[inline(always)]
    pub fn lerp(self, target: Vec3, t: f32) -> Vec3 {
        Vec3::new(
            self.x + (target.x - self.x) * t,
            self.y + (target.y - self.y) * t,
            self.z + (target.z - self.z) * t,
        )
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl AddAssign for Vec3 {
    #[inline(always)]
    fn add_assign(&mut self, o: Vec3) {
        self.x += o.x;
        self.y += o.y;
        self.z += o.z;
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl SubAssign for Vec3 {
    #[inline(always)]
    fn sub_assign(&mut self, o: Vec3) {
        self.x -= o.x;
        self.y -= o.y;
        self.z -= o.z;
    }
}
impl Mul<f32> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn mul(self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}
impl Neg for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IVec3 {
    #[inline(always)]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        IVec3 { x, y, z }
    }
}

impl Add for IVec3 {
    type Output = IVec3;
    #[inline(always)]
    fn add(self, o: IVec3) -> IVec3 {
        IVec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for IVec3 {
    type Output = IVec3;
    #[inline(always)]
    fn sub(self, o: IVec3) -> IVec3 {
        IVec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

/// Column-major 4x4 matrix (OpenGL-style memory layout), stored as 16 f32s.
#[derive(Copy, Clone, Debug)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Default for Mat4 {
    #[inline(always)]
    fn default() -> Self {
        Self::identity()
    }
}

impl Mat4 {
    #[inline(always)]
    pub fn identity() -> Self {
        let mut m = [0.0f32; 16];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[15] = 1.0;
        Mat4 { m }
    }

    #[inline(always)]
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::identity().m;
        m[12] = x;
        m[13] = y;
        m[14] = z;
        Mat4 { m }
    }

    /// Right-handed perspective projection matrix (like gluPerspective).
    pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y_rad / 2.0).tan();
        let mut m = [0.0f32; 16];
        m[0] = f / aspect;
        m[5] = f;
        m[10] = (far + near) / (near - far);
        m[11] = -1.0;
        m[14] = (2.0 * far * near) / (near - far);
        Mat4 { m }
    }

    /// Right-handed look-at view matrix.
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalized(); // forward
        let s = f.cross(up).normalized(); // right
        let u = s.cross(f); // recomputed up

        let mut m = [0.0f32; 16];
        // Column-major layout
        m[0] = s.x;
        m[4] = s.y;
        m[8] = s.z;
        m[12] = -s.dot(eye);

        m[1] = u.x;
        m[5] = u.y;
        m[9] = u.z;
        m[13] = -u.dot(eye);

        m[2] = -f.x;
        m[6] = -f.y;
        m[10] = -f.z;
        m[14] = f.dot(eye);

        m[3] = 0.0;
        m[7] = 0.0;
        m[11] = 0.0;
        m[15] = 1.0;

        Mat4 { m }
    }

    /// Multiply two column-major matrices: result = self * other
    #[inline]
    pub fn mul(&self, o: &Mat4) -> Mat4 {
        let a = &self.m;
        let b = &o.m;
        let mut r = [0.0f32; 16];
        for col in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += a[k * 4 + row] * b[col * 4 + k];
                }
                r[col * 4 + row] = sum;
            }
        }
        Mat4 { m: r }
    }

    /// Transform a point (x,y,z,1) and return the resulting homogeneous vec4.
    #[inline(always)]
    pub fn transform_point(&self, p: Vec3) -> [f32; 4] {
        let m = &self.m;
        let x = p.x;
        let y = p.y;
        let z = p.z;
        [
            m[0] * x + m[4] * y + m[8] * z + m[12],
            m[1] * x + m[5] * y + m[9] * z + m[13],
            m[2] * x + m[6] * y + m[10] * z + m[14],
            m[3] * x + m[7] * y + m[11] * z + m[15],
        ]
    }

    /// Invert this 4x4 column-major matrix. Returns None if determinant is zero.
    pub fn inverse(&self) -> Option<Self> {
        let m = &self.m;
        let mut inv = [0.0f32; 16];

        inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
            + m[9] * m[7] * m[14] + m[13] * m[6] * m[11] - m[13] * m[7] * m[10];
        inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
            - m[8] * m[7] * m[14] - m[12] * m[6] * m[11] + m[12] * m[7] * m[10];
        inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
            + m[8] * m[7] * m[13] + m[12] * m[5] * m[11] - m[12] * m[7] * m[9];
        inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
            - m[8] * m[6] * m[13] - m[12] * m[5] * m[10] + m[12] * m[6] * m[9];

        inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
            - m[9] * m[3] * m[14] - m[13] * m[2] * m[11] + m[13] * m[3] * m[10];
        inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
            + m[8] * m[3] * m[14] + m[12] * m[2] * m[11] - m[12] * m[3] * m[10];
        inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
            - m[8] * m[3] * m[13] - m[12] * m[1] * m[11] + m[12] * m[3] * m[9];
        inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
            + m[8] * m[2] * m[13] + m[12] * m[1] * m[10] - m[12] * m[2] * m[9];

        inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
            + m[5] * m[3] * m[14] + m[13] * m[2] * m[7] - m[13] * m[3] * m[6];
        inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
            - m[4] * m[3] * m[14] - m[12] * m[2] * m[7] + m[12] * m[3] * m[6];
        inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
            + m[4] * m[3] * m[13] + m[12] * m[1] * m[7] - m[12] * m[3] * m[5];
        inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
            - m[4] * m[2] * m[13] - m[12] * m[1] * m[6] + m[12] * m[2] * m[5];

        inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
            - m[5] * m[3] * m[10] - m[9] * m[2] * m[7] + m[9] * m[3] * m[6];
        inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
            + m[4] * m[3] * m[10] + m[8] * m[2] * m[7] - m[8] * m[3] * m[6];
        inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
            - m[4] * m[3] * m[9] - m[8] * m[1] * m[7] + m[8] * m[3] * m[5];
        inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
            + m[4] * m[2] * m[9] + m[8] * m[1] * m[6] - m[8] * m[2] * m[5];

        let det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];
        if det.abs() < 1e-8 {
            return None;
        }

        let inv_det = 1.0 / det;
        for val in &mut inv {
            *val *= inv_det;
        }

        Some(Mat4 { m: inv })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let sum = v1 + v2;
        assert_eq!(sum, Vec3::new(5.0, 7.0, 9.0));

        let dot = v1.dot(v2);
        assert_eq!(dot, 1.0 * 4.0 + 2.0 * 5.0 + 3.0 * 6.0);

        let cross = Vec3::new(1.0, 0.0, 0.0).cross(Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(cross, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_ivec3_conversion() {
        let v = Vec3::new(1.8, -2.3, 0.0);
        let iv = v.floor_to_ivec();
        assert_eq!(iv, IVec3::new(1, -3, 0));
    }

    #[test]
    fn test_mat4_identity_mul() {
        let id = Mat4::identity();
        let pt = Vec3::new(3.0, 4.0, 5.0);
        let transformed = id.transform_point(pt);
        assert_eq!(transformed, [3.0, 4.0, 5.0, 1.0]);
    }

    #[test]
    fn test_mat4_inverse() {
        let t = Mat4::translation(2.0, 3.0, 4.0);
        let inv = t.inverse().expect("Inverse exists");
        let combined = t.mul(&inv);
        for row in 0..4 {
            for col in 0..4 {
                let expected = if row == col { 1.0 } else { 0.0 };
                let val = combined.m[col * 4 + row];
                assert!((val - expected).abs() < 1e-5, "Expected {expected} at ({row}, {col}), got {val}");
            }
        }
    }
}
