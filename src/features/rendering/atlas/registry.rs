//! Procedural texture ID enum, array-layer mapping, and `TextureAtlas` registry (`222` built-ins + `34` mod slots).

use super::types::{speckled, Texture};
use super::terrain::{bedrock, dirt, grass_side, grass_side_autumn, grass_side_cherry, grass_side_savanna, grass_side_swamp, grass_side_taiga, grass_top, grass_top_autumn, grass_top_cherry, grass_top_savanna, grass_top_swamp, grass_top_taiga, lava, mud, sand, snow, stone, water};
use super::terrain_mineral::{clay, cobblestone, gravel, mossy_cobblestone, obsidian, sandstone_bottom, sandstone_side, sandstone_top, smooth_stone};
use super::foliage::{autumn_leaves, cactus_side, cactus_top, cherry_leaves, kelp_texture, leaves, leaves_savanna, leaves_swamp, sapling, seagrass_texture, snow_leaves, wood_side, wood_top};
use super::ores_blocks::{brick_item, bricks, clay_ball, coal_block, coal_ore, diamond, diamond_block, diamond_ore, emerald, emerald_block, emerald_ore, flint, gold_block, gold_ore, iron_block, iron_ore};
use super::blocks_craft::{bookshelf_side, crafting_table_front, crafting_table_side, crafting_table_top, door_bottom, door_top, glass, planks, tnt_side, tnt_top};
use super::blocks_containers::{bed_side, bed_top, chest_front, chest_side, chest_top, destroy_stage, entity_shadow, furnace_front, furnace_front_lit, furnace_side, furnace_top, ladder, torch};
use super::ruststone::{button_texture, lever_texture, pressure_plate_texture, ruststone_block, ruststone_dust, ruststone_lamp, ruststone_ore, ruststone_wire, wool_texture};
use super::farming::{baked_potato, bone_meal, bread, carrot, carrot_crop, farmland, farmland_side, farmland_wet, potato, potato_crop, wheat, wheat_crop, wheat_seeds};
use super::items_food::{apple, cooked_beef, cooked_chicken, cooked_cod_texture, cooked_porkchop, cooked_salmon_texture, dried_kelp_texture, pufferfish_texture, raw_beef, raw_chicken, raw_cod_texture, raw_porkchop, raw_salmon_texture, tropical_fish_texture};
use super::items_gear::{bone, diamond_armor, feather, fishing_rod_texture, gold_armor, gold_ingot, gold_ounce, gunpowder, iron_armor, iron_ingot, iron_ounce, leather_armor, shears_texture, spider_eye_texture, string_item};
use super::spawn_eggs::{chicken_spawn_egg, cow_spawn_egg, dolphin_spawn_egg_texture, fish_spawn_egg_texture, iron_golem_spawn_egg, pig_spawn_egg, sheep_spawn_egg, spider_spawn_egg, villager_spawn_egg};
use super::mobs_passive::{chicken_beak, chicken_face, chicken_feathers, chicken_leg, chicken_wattle, cod_skin, cow_face, cow_horns, cow_skin, cow_snout, cow_udder, dolphin_belly, dolphin_skin, pig_face, pig_skin, pig_snout, pufferfish_skin, salmon_skin, sheep_face, sheep_skin, tropical_fish_skin};
use super::mobs_hostile::{player_face, player_hair, player_pants, player_shirt, player_shirt_front, player_shoes, player_skin, reaper_face, reaper_skin, skeleton_body, skeleton_bone, skeleton_face, spider_face, spider_skin, zombie_face, zombie_pants, zombie_shirt, zombie_skin};
use super::mobs_npcs::{iron_golem_body, iron_golem_head, villager_arms, villager_face, villager_robe, villager_robe_blacksmith, villager_robe_butcher, villager_robe_cleric, villager_robe_librarian, villager_skin};

pub const BUILTIN_TEXTURE_COUNT: usize = 222;
/// Must satisfy: BUILTIN_TEXTURE_COUNT + CUSTOM_TEXTURE_LAYERS <= 256
/// (wgpu minimum guaranteed max_texture_array_layers on all Vulkan adapters).
/// 222 + 34 = 256 exactly.
pub const CUSTOM_TEXTURE_LAYERS: u32 = 34;

macro_rules! define_builtin_textures {
    (|$fancy:ident| $(($layer:literal, $variant:ident, $gen:expr)),* $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum TextureId {
            $($variant,)*
            /// Custom modded texture (index 0..=33)
            Custom(u8),
        }

        impl TextureId {
            #[inline(always)]
            pub const fn layer(self) -> u32 {
                match self {
                    $(Self::$variant => $layer,)*
                    Self::Custom(idx) => {
                        let clamped = if (idx as u32) > (CUSTOM_TEXTURE_LAYERS - 1) {
                            CUSTOM_TEXTURE_LAYERS - 1
                        } else {
                            idx as u32
                        };
                        (BUILTIN_TEXTURE_COUNT as u32) + clamped
                    }
                }
            }

            #[inline(always)]
            pub const fn from_layer(layer: u32) -> Self {
                match layer {
                    $($layer => Self::$variant,)*
                    l if l >= (BUILTIN_TEXTURE_COUNT as u32)
                        && l < (BUILTIN_TEXTURE_COUNT as u32) + CUSTOM_TEXTURE_LAYERS =>
                    {
                        Self::Custom((l - (BUILTIN_TEXTURE_COUNT as u32)) as u8)
                    }
                    _ => Self::CraftingTableFront,
                }
            }
        }

        pub struct TextureAtlas {
            pub builtin: Box<[Texture; BUILTIN_TEXTURE_COUNT]>,
            pub custom: Vec<Texture>,
        }

        impl TextureAtlas {
            pub fn generate() -> Self {
                Self::generate_with_fancy(true)
            }

            pub fn generate_with_fancy($fancy: bool) -> Self {
                Self {
                    builtin: Box::new([$($gen),*]),
                    custom: crate::engine::modding::generate_mod_textures(),
                }
            }

            #[inline(always)]
            pub fn get(&self, id: TextureId) -> &Texture {
                match id {
                    TextureId::Custom(idx) => self
                        .custom
                        .get(idx as usize)
                        .unwrap_or(&self.builtin[TextureId::Stone.layer() as usize]),
                    _ => &self.builtin[id.layer() as usize],
                }
            }

            #[inline(always)]
            pub fn texture_for(&self, id: TextureId) -> &Texture {
                self.get(id)
            }
        }
    };
}

define_builtin_textures! {
    |fancy_leaves|
    (0, GrassTop, grass_top()),
    (1, GrassSide, grass_side()),
    (2, Dirt, dirt()),
    (3, Stone, stone()),
    (4, Sand, sand()),
    (5, WoodTop, wood_top()),
    (6, WoodSide, wood_side()),
    (7, Leaves, leaves(fancy_leaves)),
    (8, Water, water()),
    (9, Snow, snow()),
    (10, CactusTop, cactus_top()),
    (11, CactusSide, cactus_side()),
    (12, CoalOre, coal_ore()),
    (13, IronOre, iron_ore()),
    (14, Bedrock, bedrock()),
    (15, Lava, lava()),
    (16, SnowLeaves, snow_leaves(fancy_leaves)),
    (17, Planks, planks()),
    (18, Door, door_bottom()),
    (19, Glass, glass()),
    (20, Ice, speckled([0.70, 0.88, 1.0], 0.08, 903)),
    (21, TntTop, tnt_top()),
    (22, TntSide, tnt_side()),
    (23, PigSkin, pig_skin()),
    (24, PigFace, pig_face()),
    (25, PigSnout, pig_snout()),
    (26, CowSkin, cow_skin()),
    (27, CowFace, cow_face()),
    (28, CowSnout, cow_snout()),
    (29, CowHorns, cow_horns()),
    (30, CowUdder, cow_udder()),
    (31, DoorTop, door_top()),
    (32, DestroyStage0, destroy_stage(0)),
    (33, DestroyStage1, destroy_stage(1)),
    (34, DestroyStage2, destroy_stage(2)),
    (35, DestroyStage3, destroy_stage(3)),
    (36, DestroyStage4, destroy_stage(4)),
    (37, DestroyStage5, destroy_stage(5)),
    (38, DestroyStage6, destroy_stage(6)),
    (39, DestroyStage7, destroy_stage(7)),
    (40, DestroyStage8, destroy_stage(8)),
    (41, DestroyStage9, destroy_stage(9)),
    (42, CraftingTableTop, crafting_table_top()),
    (43, CraftingTableSide, crafting_table_side()),
    (44, CraftingTableFront, crafting_table_front()),
    (45, DamageFlash, speckled([0.95, 0.05, 0.05], 0.0, 0)),
    (46, UnderwaterTint, speckled([0.05, 0.25, 0.75], 0.0, 0)),
    (47, VignetteTint, speckled([0.85, 0.02, 0.02], 0.0, 0)),
    (48, LavaTint, speckled([0.95, 0.25, 0.02], 0.0, 0)),
    (49, CherryLeaves, cherry_leaves(fancy_leaves)),
    (50, AutumnLeaves, autumn_leaves(fancy_leaves)),
    (51, Mud, mud()),
    (52, Torch, torch()),
    (53, EntityShadow, entity_shadow()),
    (54, GrassTopSwamp, grass_top_swamp()),
    (55, GrassSideSwamp, grass_side_swamp()),
    (56, LeavesSwamp, leaves_swamp(fancy_leaves)),
    (57, GrassTopSavanna, grass_top_savanna()),
    (58, GrassSideSavanna, grass_side_savanna()),
    (59, LeavesSavanna, leaves_savanna(fancy_leaves)),
    (60, GrassTopCherry, grass_top_cherry()),
    (61, GrassSideCherry, grass_side_cherry()),
    (62, GrassTopAutumn, grass_top_autumn()),
    (63, GrassSideAutumn, grass_side_autumn()),
    (64, GrassTopTaiga, grass_top_taiga()),
    (65, GrassSideTaiga, grass_side_taiga()),
    (66, ChestTop, chest_top()),
    (67, ChestSide, chest_side()),
    (68, ChestFront, chest_front()),
    (69, FurnaceTop, furnace_top()),
    (70, FurnaceSide, furnace_side()),
    (71, FurnaceFront, furnace_front()),
    (72, FurnaceFrontLit, furnace_front_lit()),
    (73, IronBlock, iron_block()),
    (74, GoldBlock, gold_block()),
    (75, GoldOre, gold_ore()),
    (76, CoalBlock, coal_block()),
    (77, RawChicken, raw_chicken()),
    (78, CookedChicken, cooked_chicken()),
    (79, Feather, feather()),
    (80, ChickenFeathers, chicken_feathers()),
    (81, ChickenFace, chicken_face()),
    (82, ChickenBeak, chicken_beak()),
    (83, ChickenWattle, chicken_wattle()),
    (84, ChickenLeg, chicken_leg()),
    (85, PigSpawnEgg, pig_spawn_egg()),
    (86, CowSpawnEgg, cow_spawn_egg()),
    (87, ChickenSpawnEgg, chicken_spawn_egg()),
    (88, RawPorkchop, raw_porkchop()),
    (89, CookedPorkchop, cooked_porkchop()),
    (90, RawBeef, raw_beef()),
    (91, CookedBeef, cooked_beef()),
    (92, Apple, apple()),
    (93, IronIngot, iron_ingot()),
    (94, GoldIngot, gold_ingot()),
    (95, IronOunce, iron_ounce()),
    (96, GoldOunce, gold_ounce()),
    (97, Bone, bone()),
    (98, PlayerSkin, player_skin()),
    (99, PlayerFace, player_face()),
    (100, PlayerHair, player_hair()),
    (101, PlayerShirt, player_shirt()),
    (102, PlayerShirtFront, player_shirt_front()),
    (103, PlayerPants, player_pants()),
    (104, PlayerShoes, player_shoes()),
    (105, DiamondOre, diamond_ore()),
    (106, DiamondBlock, diamond_block()),
    (107, Diamond, diamond()),
    (108, BedTop, bed_top()),
    (109, BedSide, bed_side()),
    (110, Ladder, ladder()),
    (111, ZombieSkin, zombie_skin()),
    (112, ZombieFace, zombie_face()),
    (113, ZombieShirt, zombie_shirt()),
    (114, ZombiePants, zombie_pants()),
    (115, SkeletonBone, skeleton_bone()),
    (116, SkeletonFace, skeleton_face()),
    (117, SkeletonBody, skeleton_body()),
    (118, ReaperSkin, reaper_skin()),
    (119, ReaperFace, reaper_face()),
    (120, Gunpowder, gunpowder()),
    (121, Farmland, farmland()),
    (122, FarmlandWet, farmland_wet()),
    (123, FarmlandSide, farmland_side()),
    (124, WheatSeeds, wheat_seeds()),
    (125, WheatCrop, wheat_crop()),
    (126, Wheat, wheat()),
    (127, Bread, bread()),
    (128, Carrot, carrot()),
    (129, CarrotCrop, carrot_crop()),
    (130, Potato, potato()),
    (131, PotatoCrop, potato_crop()),
    (132, BakedPotato, baked_potato()),
    (133, Sapling, sapling()),
    (134, BoneMeal, bone_meal()),
    (135, RustStoneOre, ruststone_ore()),
    (136, RustStone, ruststone_dust()),
    (137, RustStoneWire, ruststone_wire(false)),
    (138, RustStoneWireLit, ruststone_wire(true)),
    (139, RustStoneBlock, ruststone_block()),
    (140, RustStoneLamp, ruststone_lamp(false)),
    (141, RustStoneLampLit, ruststone_lamp(true)),
    (142, Lever, lever_texture()),
    (143, Button, button_texture()),
    (144, PressurePlate, pressure_plate_texture()),
    (145, WhiteWool, wool_texture([0.91, 0.92, 0.92], 101)),
    (146, OrangeWool, wool_texture([0.94, 0.46, 0.07], 102)),
    (147, MagentaWool, wool_texture([0.74, 0.27, 0.70], 103)),
    (148, LightBlueWool, wool_texture([0.23, 0.69, 0.85], 104)),
    (149, YellowWool, wool_texture([0.97, 0.78, 0.15], 105)),
    (150, LimeWool, wool_texture([0.44, 0.73, 0.10], 106)),
    (151, PinkWool, wool_texture([0.93, 0.55, 0.67], 107)),
    (152, GrayWool, wool_texture([0.24, 0.27, 0.28], 108)),
    (153, LightGrayWool, wool_texture([0.56, 0.56, 0.53], 109)),
    (154, CyanWool, wool_texture([0.08, 0.54, 0.57], 110)),
    (155, PurpleWool, wool_texture([0.47, 0.16, 0.67], 111)),
    (156, BlueWool, wool_texture([0.21, 0.22, 0.62], 112)),
    (157, BrownWool, wool_texture([0.45, 0.28, 0.16], 113)),
    (158, GreenWool, wool_texture([0.33, 0.43, 0.11], 114)),
    (159, RedWool, wool_texture([0.63, 0.15, 0.13], 115)),
    (160, BlackWool, wool_texture([0.08, 0.08, 0.10], 116)),
    (161, SheepSkin, sheep_skin()),
    (162, SheepFace, sheep_face()),
    (163, Shears, shears_texture()),
    (164, SheepSpawnEgg, sheep_spawn_egg()),
    (165, Kelp, kelp_texture()),
    (166, Seagrass, seagrass_texture()),
    (167, RawCod, raw_cod_texture()),
    (168, CookedCod, cooked_cod_texture()),
    (169, RawSalmon, raw_salmon_texture()),
    (170, CookedSalmon, cooked_salmon_texture()),
    (171, Pufferfish, pufferfish_texture()),
    (172, TropicalFish, tropical_fish_texture()),
    (173, DriedKelp, dried_kelp_texture()),
    (174, DolphinSpawnEgg, dolphin_spawn_egg_texture()),
    (175, FishSpawnEgg, fish_spawn_egg_texture()),
    (176, FishingRod, fishing_rod_texture()),
    (177, DolphinSkin, dolphin_skin()),
    (178, DolphinBelly, dolphin_belly()),
    (179, CodSkin, cod_skin()),
    (180, SalmonSkin, salmon_skin()),
    (181, PufferfishSkin, pufferfish_skin()),
    (182, TropicalFishSkin, tropical_fish_skin()),
    (183, Emerald, emerald()),
    (184, EmeraldOre, emerald_ore()),
    (185, EmeraldBlock, emerald_block()),
    (186, VillagerSpawnEgg, villager_spawn_egg()),
    (187, VillagerSkin, villager_skin()),
    (188, VillagerFace, villager_face()),
    (189, VillagerRobe, villager_robe()),
    (190, VillagerArms, villager_arms()),
    (191, VillagerRobeLibrarian, villager_robe_librarian()),
    (192, VillagerRobeBlacksmith, villager_robe_blacksmith()),
    (193, VillagerRobeCleric, villager_robe_cleric()),
    (194, VillagerRobeButcher, villager_robe_butcher()),
    (195, IronGolemBody, iron_golem_body()),
    (196, IronGolemHead, iron_golem_head()),
    (197, IronGolemSpawnEgg, iron_golem_spawn_egg()),
    (198, Clay, clay()),
    (199, ClayBall, clay_ball()),
    (200, Brick, brick_item()),
    (201, Bricks, bricks()),
    (202, Gravel, gravel()),
    (203, Flint, flint()),
    (204, Cobblestone, cobblestone()),
    (205, MossyCobblestone, mossy_cobblestone()),
    (206, SandstoneTop, sandstone_top()),
    (207, SandstoneSide, sandstone_side()),
    (208, SandstoneBottom, sandstone_bottom()),
    (209, SmoothStone, smooth_stone()),
    (210, Obsidian, obsidian()),
    (211, BookshelfSide, bookshelf_side()),
    (212, BookshelfTop, planks()),
    (213, SpiderSkin, spider_skin()),
    (214, SpiderFace, spider_face()),
    (215, SpiderSpawnEgg, spider_spawn_egg()),
    (216, StringItem, string_item()),
    (217, SpiderEye, spider_eye_texture()),
    (218, LeatherArmor, leather_armor()),
    (219, IronArmor, iron_armor()),
    (220, GoldArmor, gold_armor()),
    (221, DiamondArmor, diamond_armor()),
}

impl TextureId {
    #[inline(always)]
    pub fn is_destroy_stage(self) -> bool {
        matches!(
            self,
            TextureId::DestroyStage0
                | TextureId::DestroyStage1
                | TextureId::DestroyStage2
                | TextureId::DestroyStage3
                | TextureId::DestroyStage4
                | TextureId::DestroyStage5
                | TextureId::DestroyStage6
                | TextureId::DestroyStage7
                | TextureId::DestroyStage8
                | TextureId::DestroyStage9
        )
    }

    pub fn destroy_stage(stage: usize) -> Self {
        match stage {
            0 => TextureId::DestroyStage0,
            1 => TextureId::DestroyStage1,
            2 => TextureId::DestroyStage2,
            3 => TextureId::DestroyStage3,
            4 => TextureId::DestroyStage4,
            5 => TextureId::DestroyStage5,
            6 => TextureId::DestroyStage6,
            7 => TextureId::DestroyStage7,
            8 => TextureId::DestroyStage8,
            _ => TextureId::DestroyStage9,
        }
    }
}
