pub mod math;
pub mod noise;

pub use math::{IVec3, Mat4, Vec3};
pub use noise::{fractal_noise2, fractal_noise3, value_noise2, value_noise3};
