//! Modular block and voxel registry system.
//!
//! Provides extensible metadata attributes (hardness, opacity, luminance, textures,
//! collision flags, and custom properties) allowing new voxel types and materials
//! to be registered dynamically without modifying core engine loops.

use crate::world::block::{BlockId, Face};
use crate::graphics::atlas::TextureId;

/// Rendering and collision properties of a voxel block.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockProperties {
    /// Numerical identifier.
    pub id: BlockId,
    /// Human-readable localized name for UI and tooltips.
    pub name: &'static str,
    /// Solid collision bounding box enabled.
    pub is_solid: bool,
    /// Fully blocks light transmission and face-culls adjacent blocks.
    pub is_opaque: bool,
    /// Fluid mechanics (flowing water/lava).
    pub is_fluid: bool,
    /// Light emission level (0 to 15).
    pub light_emission: u8,
    /// Mining hardness in seconds with bare hands.
    pub hardness: f32,
    /// Foliage/leaf rendering characteristics (cutout transparency).
    pub is_leaf: bool,
    /// Flammability in contact with lava.
    pub is_flammable: bool,
}

impl Default for BlockProperties {
    fn default() -> Self {
        Self {
            id: BlockId::Air,
            name: "Air",
            is_solid: false,
            is_opaque: false,
            is_fluid: false,
            light_emission: 0,
            hardness: 0.0,
            is_leaf: false,
            is_flammable: false,
        }
    }
}

/// Registry for block properties and dynamic texture mapping.
#[derive(Clone, Debug, Default)]
pub struct BlockRegistry {
    properties: Vec<BlockProperties>,
}

impl BlockRegistry {
    /// Creates a new registry initialized with default voxel definitions.
    pub fn new() -> Self {
        let mut registry = Self {
            properties: Vec::with_capacity(256),
        };
        registry.register_defaults();
        registry
    }

    /// Registers default vanilla block definitions.
    fn register_defaults(&mut self) {
        for id_val in 0..=43u8 {
            let id = BlockId::from_u8(id_val);
            let is_fluid = matches!(id, BlockId::Water | BlockId::Lava);
            let is_flammable = matches!(id, BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::Wood | BlockId::Planks);
            self.register(BlockProperties {
                id,
                name: id.name(),
                is_solid: id.is_solid(),
                is_opaque: id.is_opaque(),
                is_fluid,
                light_emission: if id == BlockId::Torch || id == BlockId::Lava { 14 } else { 0 },
                hardness: if id == BlockId::Bedrock { -1.0 } else { 1.5 },
                is_leaf: id.is_leaf(),
                is_flammable,
            });
        }
    }

    /// Registers a new block definition in the registry.
    pub fn register(&mut self, props: BlockProperties) {
        let idx = props.id.to_u8() as usize;
        if idx >= self.properties.len() {
            self.properties.resize_with(idx + 1, BlockProperties::default);
        }
        self.properties[idx] = props;
    }

    /// Retrieves properties for a given `BlockId`.
    #[inline(always)]
    pub fn get(&self, id: BlockId) -> &BlockProperties {
        let idx = id.to_u8() as usize;
        self.properties.get(idx).unwrap_or(&self.properties[0])
    }

    /// Returns texture mapping for a given block face.
    #[inline(always)]
    pub fn get_face_texture(&self, id: BlockId, face: Face) -> TextureId {
        id.texture_id(face)
    }
}
