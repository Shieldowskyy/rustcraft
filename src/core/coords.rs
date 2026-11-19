//! Strongly-typed voxel coordinate spaces (`WorldPos`, `ChunkPos`, `LocalPos`) and `BlockFace`.
//! Pure value types with checked/bitwise chunk indexing; modifies no global state.
//! Converts between world-space integers, chunk columns (`16x64x16`), and flat voxel offsets.

use crate::core::math::{IVec3, Vec3};

pub const CHUNK_WIDTH: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 64;
pub const CHUNK_DEPTH: i32 = 16;

/// Global integer block coordinate in world space (`+Y` up, `-Z` north, `+X` east).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct WorldPos(pub IVec3);

impl WorldPos {
    #[inline(always)]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    #[inline(always)]
    pub fn from_vec3(v: Vec3) -> Self {
        Self(v.floor_to_ivec())
    }

    #[inline(always)]
    pub const fn x(self) -> i32 {
        self.0.x
    }

    #[inline(always)]
    pub const fn y(self) -> i32 {
        self.0.y
    }

    #[inline(always)]
    pub const fn z(self) -> i32 {
        self.0.z
    }

    #[inline(always)]
    pub const fn as_ivec3(self) -> IVec3 {
        self.0
    }

    /// Maps world `(x, z)` to its enclosing `16x16` chunk column using arithmetic shift.
    #[inline(always)]
    pub const fn chunk_pos(self) -> ChunkPos {
        ChunkPos {
            x: self.0.x >> 4,
            z: self.0.z >> 4,
        }
    }

    /// Extracts local `(x, y, z)` inside the chunk, returning `None` if `y` is outside `0..64`.
    #[inline(always)]
    pub const fn local_pos(self) -> Option<LocalPos> {
        if self.0.y < 0 || self.0.y >= CHUNK_HEIGHT {
            None
        } else {
            Some(LocalPos {
                x: (self.0.x & 15) as u8,
                y: self.0.y as u8,
                z: (self.0.z & 15) as u8,
            })
        }
    }

    /// Returns horizontal `(lx, lz)` in `0..16` regardless of `y` bounds.
    #[inline(always)]
    pub const fn local_xz(self) -> (u8, u8) {
        ((self.0.x & 15) as u8, (self.0.z & 15) as u8)
    }

    #[inline(always)]
    pub const fn offset(self, face: BlockFace) -> Self {
        let n = face.normal();
        Self(IVec3::new(self.0.x + n.x, self.0.y + n.y, self.0.z + n.z))
    }
}

impl From<IVec3> for WorldPos {
    #[inline(always)]
    fn from(v: IVec3) -> Self {
        Self(v)
    }
}

impl From<WorldPos> for IVec3 {
    #[inline(always)]
    fn from(p: WorldPos) -> Self {
        p.0
    }
}

/// Chunk column coordinate in chunk-grid space (`1 chunk = 16x64x16` blocks).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    #[inline(always)]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    #[inline(always)]
    pub const fn from_world(wx: i32, wz: i32) -> Self {
        Self {
            x: wx >> 4,
            z: wz >> 4,
        }
    }

    #[inline(always)]
    pub const fn to_world_origin(self) -> WorldPos {
        WorldPos::new(self.x * CHUNK_WIDTH, 0, self.z * CHUNK_DEPTH)
    }

    #[inline(always)]
    pub const fn distance_sq(self, other: Self) -> i32 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        dx * dx + dz * dz
    }
}

impl From<(i32, i32)> for ChunkPos {
    #[inline(always)]
    fn from((x, z): (i32, i32)) -> Self {
        Self { x, z }
    }
}

impl From<ChunkPos> for (i32, i32) {
    #[inline(always)]
    fn from(pos: ChunkPos) -> Self {
        (pos.x, pos.z)
    }
}

/// Validated block coordinate inside a single `16x64x16` chunk.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct LocalPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl LocalPos {
    #[inline(always)]
    pub const fn new_checked(x: i32, y: i32, z: i32) -> Option<Self> {
        if (x as u32) < CHUNK_WIDTH as u32
            && (y as u32) < CHUNK_HEIGHT as u32
            && (z as u32) < CHUNK_DEPTH as u32
        {
            Some(Self {
                x: x as u8,
                y: y as u8,
                z: z as u8,
            })
        } else {
            None
        }
    }

    /// Flat array index matching `(x << 10) | (y << 4) | z` for `Chunk::blocks`.
    #[inline(always)]
    pub const fn chunk_index(self) -> usize {
        ((self.x as usize) << 10) | ((self.y as usize) << 4) | (self.z as usize)
    }
}

/// Canonical six cardinal block faces in RustCraft's right-handed world space.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Top,
    Bottom,
    North, // -Z
    South, // +Z
    East,  // +X
    West,  // -X
}

impl BlockFace {
    pub const ALL: [BlockFace; 6] = [
        BlockFace::Top,
        BlockFace::Bottom,
        BlockFace::North,
        BlockFace::South,
        BlockFace::East,
        BlockFace::West,
    ];

    #[inline(always)]
    pub const fn shade(self) -> f32 {
        match self {
            BlockFace::Top => 1.0,
            BlockFace::North | BlockFace::South => 0.85,
            BlockFace::East | BlockFace::West => 0.7,
            BlockFace::Bottom => 0.5,
        }
    }

    #[inline(always)]
    pub const fn normal(self) -> IVec3 {
        match self {
            BlockFace::Top => IVec3::new(0, 1, 0),
            BlockFace::Bottom => IVec3::new(0, -1, 0),
            BlockFace::North => IVec3::new(0, 0, -1),
            BlockFace::South => IVec3::new(0, 0, 1),
            BlockFace::East => IVec3::new(1, 0, 0),
            BlockFace::West => IVec3::new(-1, 0, 0),
        }
    }

    #[inline(always)]
    pub const fn opposite(self) -> Self {
        match self {
            BlockFace::Top => BlockFace::Bottom,
            BlockFace::Bottom => BlockFace::Top,
            BlockFace::North => BlockFace::South,
            BlockFace::South => BlockFace::North,
            BlockFace::East => BlockFace::West,
            BlockFace::West => BlockFace::East,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negative_world_to_chunk_and_local() {
        let pos = WorldPos::new(-1, 12, -17);
        assert_eq!(pos.chunk_pos(), ChunkPos::new(-1, -2));
        let local = pos.local_pos().expect("y=12 is inside 0..64");
        assert_eq!(local, LocalPos { x: 15, y: 12, z: 15 });
        assert_eq!(local.chunk_index(), (15 << 10) | (12 << 4) | 15);
    }

    #[test]
    fn test_local_pos_out_of_y_bounds() {
        assert!(WorldPos::new(0, -1, 0).local_pos().is_none());
        assert!(WorldPos::new(0, CHUNK_HEIGHT, 0).local_pos().is_none());
        assert!(LocalPos::new_checked(16, 0, 0).is_none());
    }

    #[test]
    fn test_face_normals_and_opposites() {
        for face in BlockFace::ALL {
            assert_eq!(face.opposite().opposite(), face);
            let n = face.normal();
            let opp = face.opposite().normal();
            assert_eq!(n.x + opp.x, 0);
            assert_eq!(n.y + opp.y, 0);
            assert_eq!(n.z + opp.z, 0);
        }
    }
}

