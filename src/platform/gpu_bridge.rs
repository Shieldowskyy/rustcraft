//! Unconditional hardware GPU bridge for `features::rendering` without platform `#[cfg]`.
//! Wraps optional `platform::wgpu::VulkanContext` on WGPU-enabled targets and no-ops elsewhere.

use std::collections::{HashMap, HashSet};

use crate::core::math::{Mat4, Vec3};
use crate::features::meshing::Triangle;
use crate::features::rendering::atlas::TextureAtlas;

/// Target-agnostic handle to an optional WGPU hardware world renderer.
pub struct GpuBridge {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_arch = "powerpc"),
        not(target_os = "vita"),
        not(target_os = "xbox360"),
        not(target_os = "xenon"),
        not(feature = "xbox360")
    ))]
    inner: Option<super::wgpu::VulkanContext>,
}

impl GpuBridge {
    /// Creates an inactive bridge that falls back to software/native rendering.
    pub const fn none() -> Self {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        {
            Self { inner: None }
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        {
            Self {}
        }
    }

    /// Initializes the primary/Vulkan WGPU backend on supported desktop hosts.
    pub fn new_vulkan(width: usize, height: usize, atlas: &TextureAtlas) -> Result<Self, String> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "android"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        {
            Ok(Self {
                inner: Some(super::wgpu::VulkanContext::new(width, height, atlas)?),
            })
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "android",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        {
            let _ = (width, height, atlas);
            Err("Vulkan WGPU backend is not supported on this target".to_owned())
        }
    }

    /// Initializes the OpenGL/GLES WGPU backend on desktop and Android hosts.
    pub fn new_opengl(width: usize, height: usize, atlas: &TextureAtlas) -> Result<Self, String> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        {
            Ok(Self {
                inner: Some(super::wgpu::VulkanContext::new_with_backends(
                    width,
                    height,
                    atlas,
                    Some(::wgpu::Backends::GL),
                )?),
            })
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        {
            let _ = (width, height, atlas);
            Err("OpenGL WGPU backend is not supported on this target".to_owned())
        }
    }

    #[inline(always)]
    pub fn is_active(&self) -> bool {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        {
            self.inner.is_some()
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        {
            false
        }
    }

    #[inline(always)]
    pub fn backend_name(&self) -> Option<&'static str> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        {
            self.inner.as_ref().map(|vk| vk.backend_name())
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        {
            None
        }
    }

    #[inline(always)]
    pub fn gpu_info(&self) -> Option<(&str, &str, &str)> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        {
            self.inner.as_ref().map(|vk| vk.gpu_info())
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        {
            None
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        if let Some(vk) = self.inner.as_mut() {
            vk.resize(width, height);
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        let _ = (width, height);
    }

    pub fn update_textures(&mut self, atlas: &TextureAtlas) {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        if let Some(vk) = self.inner.as_mut() {
            vk.update_textures(atlas);
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        let _ = atlas;
    }

    pub fn clear_chunk_buffers(&mut self) {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        if let Some(vk) = self.inner.as_mut() {
            vk.clear_chunk_buffers();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_world(
        &mut self,
        mesh_cache: &HashMap<(i32, i32), Vec<Triangle>>,
        loaded_keys: &[(i32, i32)],
        dirty_keys: &HashSet<(i32, i32)>,
        visible_keys: &[(i32, i32)],
        view_proj: &Mat4,
        camera: Vec3,
        pitch: f32,
        render_distance: i32,
        fog_enabled: bool,
        chunk_anim: bool,
        total_time: f32,
        chunk_spawn_times: &HashMap<(i32, i32), f32>,
        daylight: f32,
        day_fraction: f32,
        sun_dir: Vec3,
        sky_colors: ([f32; 3], [f32; 3], [f32; 3]),
        out_pixels: &mut [u32],
        out_depths: &mut [f32],
    ) -> bool {
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_arch = "powerpc"),
            not(target_os = "vita"),
            not(target_os = "xbox360"),
            not(target_os = "xenon"),
            not(feature = "xbox360")
        ))]
        if let Some(vk) = self.inner.as_mut() {
            vk.render_world(
                mesh_cache,
                loaded_keys,
                dirty_keys,
                visible_keys,
                view_proj,
                camera,
                pitch,
                render_distance,
                fog_enabled,
                chunk_anim,
                total_time,
                chunk_spawn_times,
                daylight,
                day_fraction,
                sun_dir,
                sky_colors,
                out_pixels,
                out_depths,
            );
            return true;
        }
        #[cfg(any(
            target_arch = "wasm32",
            target_arch = "powerpc",
            target_os = "vita",
            target_os = "xbox360",
            target_os = "xenon",
            feature = "xbox360"
        ))]
        let _ = (
            mesh_cache,
            loaded_keys,
            dirty_keys,
            visible_keys,
            view_proj,
            camera,
            pitch,
            render_distance,
            fog_enabled,
            chunk_anim,
            total_time,
            chunk_spawn_times,
            daylight,
            day_fraction,
            sun_dir,
            sky_colors,
            out_pixels,
            out_depths,
        );
        false
    }
}
