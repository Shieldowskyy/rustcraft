//! Linear ABGR8888 GPU texture atlas packing and tile UV lookup for PS Vita `libgxm`.
//! Copies procedural/modded `TextureAtlas` tiles into uncached GXM texture memory.

#![allow(dead_code)]

use crate::graphics::atlas::{TextureAtlas, TextureId, TEX_SIZE};

use super::ffi::*;
use super::mem::GpuMem;

// ============================================================
// 5. GPU TEXTURE ATLAS (Linear ABGR8888 Format)
// ============================================================

const ATLAS_TILES_PER_ROW: usize = 16;
const ATLAS_SIZE: usize = ATLAS_TILES_PER_ROW * TEX_SIZE; // 256x256 px

macro_rules! define_vita_atlas_tiles {
    ($(($idx:literal, $variant:ident)),* $(,)?) => {
        pub const ALL_TEXTURES: &[TextureId] = &[$(TextureId::$variant),*];

        #[inline(always)]
        const fn vita_tile_index(tex_id: TextureId) -> usize {
            match tex_id {
                $(TextureId::$variant => $idx,)*
                _ => 0,
            }
        }
    };
}

define_vita_atlas_tiles! {
    (0, GrassTop), (1, GrassSide), (2, Dirt), (3, Stone), (4, Sand),
    (5, WoodTop), (6, WoodSide), (7, Leaves), (8, Water), (9, Snow),
    (10, CactusTop), (11, CactusSide), (12, CoalOre), (13, IronOre), (14, Bedrock),
    (15, Lava), (16, SnowLeaves), (17, Planks), (18, Door), (19, DoorTop),
    (20, Glass), (21, Ice), (22, TntTop), (23, TntSide), (24, PigSkin),
    (25, PigFace), (26, PigSnout), (27, CowSkin), (28, CowFace), (29, CowSnout),
    (30, CowHorns), (31, CowUdder), (32, CraftingTableTop), (33, CraftingTableSide), (34, CraftingTableFront),
    (35, DestroyStage0), (36, DestroyStage1), (37, DestroyStage2), (38, DestroyStage3), (39, DestroyStage4),
    (40, DestroyStage5), (41, DestroyStage6), (42, DestroyStage7), (43, DestroyStage8), (44, DestroyStage9),
    (45, DamageFlash), (46, UnderwaterTint), (47, VignetteTint), (48, LavaTint), (49, CherryLeaves),
    (50, AutumnLeaves), (51, Mud), (52, Torch), (53, EntityShadow), (54, GrassTopSwamp),
    (55, GrassSideSwamp), (56, LeavesSwamp), (57, GrassTopSavanna), (58, GrassSideSavanna), (59, LeavesSavanna),
    (60, GrassTopCherry), (61, GrassSideCherry), (62, GrassTopAutumn), (63, GrassSideAutumn), (64, GrassTopTaiga),
    (65, GrassSideTaiga), (66, ChestTop), (67, ChestSide), (68, ChestFront), (69, FurnaceTop),
    (70, FurnaceSide), (71, FurnaceFront), (72, FurnaceFrontLit), (73, IronBlock), (74, GoldBlock),
    (75, GoldOre), (76, CoalBlock), (77, RawChicken), (78, CookedChicken), (79, Feather),
    (80, ChickenFeathers), (81, ChickenFace), (82, ChickenBeak), (83, ChickenWattle), (84, ChickenLeg),
    (85, PigSpawnEgg), (86, CowSpawnEgg), (87, ChickenSpawnEgg), (88, RawPorkchop), (89, CookedPorkchop),
    (90, RawBeef), (91, CookedBeef), (92, Apple), (93, IronIngot), (94, GoldIngot),
    (95, IronOunce), (96, GoldOunce), (97, Bone), (98, DiamondOre), (99, DiamondBlock),
    (100, Diamond), (101, BedTop), (102, BedSide), (103, Ladder), (104, ZombieSkin),
    (105, ZombieFace), (106, ZombieShirt), (107, ZombiePants), (108, SkeletonBone), (109, SkeletonFace),
    (110, SkeletonBody), (111, ReaperSkin), (112, ReaperFace), (113, Gunpowder), (114, WhiteWool),
    (115, OrangeWool), (116, MagentaWool), (117, LightBlueWool), (118, YellowWool), (119, LimeWool),
    (120, PinkWool), (121, GrayWool), (122, LightGrayWool), (123, CyanWool), (124, PurpleWool),
    (125, BlueWool), (126, BrownWool), (127, GreenWool), (128, RedWool), (129, BlackWool),
    (130, SheepSkin), (131, SheepFace), (132, Shears), (133, SheepSpawnEgg), (134, Kelp),
    (135, Seagrass), (136, RawCod), (137, CookedCod), (138, RawSalmon), (139, CookedSalmon),
    (140, Pufferfish), (141, TropicalFish), (142, DriedKelp), (143, DolphinSpawnEgg), (144, FishSpawnEgg),
    (145, FishingRod), (146, DolphinSkin), (147, DolphinBelly), (148, CodSkin), (149, SalmonSkin),
    (150, PufferfishSkin), (151, TropicalFishSkin), (152, Emerald), (153, EmeraldOre), (154, EmeraldBlock),
    (155, VillagerSpawnEgg), (156, VillagerSkin), (157, VillagerFace), (158, VillagerRobe), (159, VillagerArms),
    (160, VillagerRobeLibrarian), (161, VillagerRobeBlacksmith), (162, VillagerRobeCleric), (163, VillagerRobeButcher),
    (164, RustStoneWire), (165, RustStoneWireLit), (166, Lever), (167, Button),
    (168, PressurePlate), (169, WheatCrop), (170, CarrotCrop), (171, PotatoCrop),
    (172, Sapling),
}

pub(super) unsafe fn populate_atlas(pixels: *mut u32, atlas: &TextureAtlas) {
    for (tile_idx, &tex_id) in ALL_TEXTURES.iter().enumerate() {
        let tex = atlas.texture_for(tex_id);
        let tile_col = tile_idx % ATLAS_TILES_PER_ROW;
        let tile_row = tile_idx / ATLAS_TILES_PER_ROW;
        let base_x = tile_col * TEX_SIZE;
        let base_y = tile_row * TEX_SIZE;
        let is_water = tex_id == TextureId::Water;
        let is_glass = tex_id == TextureId::Glass;
        let is_damage = tex_id == TextureId::DamageFlash;
        let is_underwater = tex_id == TextureId::UnderwaterTint;
        let is_vignette = tex_id == TextureId::VignetteTint;
        let is_lava = tex_id == TextureId::LavaTint;
        let is_shadow = tex_id == TextureId::EntityShadow;
        for py in 0..TEX_SIZE {
            for px in 0..TEX_SIZE {
                let [r, g, b] = tex.texels[py * TEX_SIZE + px];
                let is_transparent = r < 0.0;
                let real_r = if is_transparent { (-r - 0.001).clamp(0.0, 1.0) } else { r.clamp(0.0, 1.0) };
                let a = if is_transparent {
                    0u32
                } else if is_shadow {
                    130u32 // Semi-transparent black entity shadow (~51% opacity)
                } else if is_water {
                    122u32 // Semi-transparent water (~48% opacity, matching WGSL 0.48)
                } else if is_glass {
                    122u32 // Semi-transparent glass (~48% opacity, matching WGSL 0.48)
                } else if is_damage {
                    85u32  // Semi-transparent damage flash (~33% opacity)
                } else if is_underwater {
                    95u32  // Semi-transparent underwater overlay (~37% opacity)
                } else if is_vignette {
                    70u32  // Semi-transparent heartbeat vignette
                } else if is_lava {
                    215u32 // Dense, glowing lava overlay (~84% opacity)
                } else {
                    255u32
                };
                let ri = (real_r * 255.0) as u32;
                let gi = (g.clamp(0.0, 1.0) * 255.0) as u32;
                let bi = (b.clamp(0.0, 1.0) * 255.0) as u32;
                // ABGR8888
                let abgr = (a << 24) | (bi << 16) | (gi << 8) | ri;
                *pixels.add((base_y + py) * ATLAS_SIZE + (base_x + px)) = abgr;
            }
        }
    }
}

pub(super) fn build_gpu_texture(atlas: &TextureAtlas) -> (GpuMem, SceGxmTexture) {
    let tex_bytes = ATLAS_SIZE * ATLAS_SIZE * 4;
    let mem = GpuMem::alloc_gpu(
        b"RC_GXM_Atlas\0",
        tex_bytes,
        SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
        SCE_GXM_MEMORY_ATTRIB_READ,
    ).expect("Failed to allocate atlas texture in CDRAM");

    let pixels = mem.ptr as *mut u32;

    unsafe {
        core::ptr::write_bytes(pixels as *mut u8, 0, tex_bytes);
        populate_atlas(pixels, atlas);
    }

    let mut texture = SceGxmTexture { _private: [0u32; 4] };
    unsafe {
        sceGxmTextureInitLinear(
            &mut texture,
            mem.ptr,
            SCE_GXM_TEXTURE_FORMAT_U8U8U8U8_ABGR,
            ATLAS_SIZE as u32,
            ATLAS_SIZE as u32,
            1,
        );
        sceGxmTextureSetMinFilter(&mut texture, SCE_GXM_TEXTURE_FILTER_POINT);
        sceGxmTextureSetMagFilter(&mut texture, SCE_GXM_TEXTURE_FILTER_POINT);
        sceGxmTextureSetUAddrMode(&mut texture, SCE_GXM_TEXTURE_ADDR_REPEAT);
        sceGxmTextureSetVAddrMode(&mut texture, SCE_GXM_TEXTURE_ADDR_REPEAT);
    }

    (mem, texture)
}

#[inline(always)]
pub(super) fn atlas_uv(tex_id: TextureId) -> (f32, f32, f32, f32) {
    let tile_idx = vita_tile_index(tex_id);
    const TILE_NORM: f32 = 1.0 / ATLAS_TILES_PER_ROW as f32;
    let col = tile_idx % ATLAS_TILES_PER_ROW;
    let row = tile_idx / ATLAS_TILES_PER_ROW;
    // Sub-texel inset (0.375 texels) prevents edge bleeding between adjacent atlas tiles
    let inset = 0.375 / ATLAS_SIZE as f32;
    let u0 = col as f32 * TILE_NORM + inset;
    let v0 = row as f32 * TILE_NORM + inset;
    let u1 = (col + 1) as f32 * TILE_NORM - inset;
    let v1 = (row + 1) as f32 * TILE_NORM - inset;
    (u0, v0, u1, v1)
}

