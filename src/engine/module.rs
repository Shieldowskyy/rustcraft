//! Pluggable engine module and lifecycle hook system.
//!
//! Enables adding custom gameplay systems, mod extensions, custom render passes,
//! audio drivers, or analytics without altering the core engine state machine.

use crate::app::{AppOutput, InputState};
use crate::world::World;
use crate::graphics::camera::Camera;

/// Context passed into engine modules during each update and render tick.
pub struct ModuleContext<'a> {
    /// Active voxel world.
    pub world: &'a mut World,
    /// Active camera instance.
    pub camera: &'a Camera,
    /// Delta time since last frame in seconds.
    pub dt: f32,
    /// Framebuffer pixel dimension (width, height).
    pub resolution: (usize, usize),
    /// Active input snapshot.
    pub input: &'a InputState,
}

/// Pluggable engine module interface.
pub trait EngineModule: Send + Sync {
    /// Name of the module for logging and diagnostics.
    fn name(&self) -> &'static str;

    /// Called once when the engine initializes.
    fn on_init(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Pre-update tick executed before main world simulation.
    fn on_pre_update(&mut self, _ctx: &mut ModuleContext) {}

    /// Post-update tick executed after main world simulation and physics.
    fn on_post_update(&mut self, _ctx: &mut ModuleContext, _output: &mut AppOutput) {}

    /// Called when the application is shutting down.
    fn on_shutdown(&mut self) {}
}

/// Manager executing registered engine modules.
#[derive(Default)]
pub struct ModuleManager {
    modules: Vec<Box<dyn EngineModule>>,
}

impl ModuleManager {
    /// Creates an empty module manager.
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    /// Registers a new engine module.
    pub fn register<M: EngineModule + 'static>(&mut self, mut module: M) -> Result<(), String> {
        module.on_init()?;
        self.modules.push(Box::new(module));
        Ok(())
    }

    /// Dispatches pre-update hook across all registered modules.
    pub fn dispatch_pre_update(&mut self, ctx: &mut ModuleContext) {
        for module in &mut self.modules {
            module.on_pre_update(ctx);
        }
    }

    /// Dispatches post-update hook across all registered modules.
    pub fn dispatch_post_update(&mut self, ctx: &mut ModuleContext, output: &mut AppOutput) {
        for module in &mut self.modules {
            module.on_post_update(ctx, output);
        }
    }

    /// Dispatches shutdown notification to all modules.
    pub fn dispatch_shutdown(&mut self) {
        for module in &mut self.modules {
            module.on_shutdown();
        }
    }
}
