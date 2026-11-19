//! Deterministic vanilla and modded loot drops emitted when a mob dies.
//! Populates `dying_drops` from a killed `Mob`'s kind, ID, wool color, and reaper fuse state.

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;

use super::kind::MobKind;
use super::manager::MobDrop;
use super::state::Mob;

pub(super) fn collect_mob_drops(m: &Mob, out: &mut Vec<MobDrop>) {
    let center = m.center_pos();
    if !m.is_baby {
        match m.kind {
            MobKind::Pig => {
                out.push(MobDrop { pos: center, block: BlockId::RawPorkchop, count: 1 + (m.id % 3) as u8 });
            }
            MobKind::Cow => {
                out.push(MobDrop { pos: center, block: BlockId::RawBeef, count: 1 + (m.id % 3) as u8 });
                let leather = (m.id % 3) as u8;
                if leather > 0 {
                    out.push(MobDrop {
                        pos: center + Vec3::new(0.05, 0.1, 0.05),
                        block: BlockId::Leather,
                        count: leather,
                    });
                }
            }
            MobKind::Sheep => {
                out.push(MobDrop { pos: center, block: BlockId::RawBeef, count: 1 + (m.id % 2) as u8 });
                if !m.is_sheared {
                    out.push(MobDrop {
                        pos: center + Vec3::new(0.05, 0.1, 0.05),
                        block: BlockId::wool_block_from_color(m.wool_color),
                        count: 1,
                    });
                }
            }
            MobKind::Chicken => {
                out.push(MobDrop { pos: center, block: BlockId::RawChicken, count: 1 + (m.id % 2) as u8 });
                out.push(MobDrop {
                    pos: center + Vec3::new(0.1, 0.1, 0.1),
                    block: BlockId::Feather,
                    count: 1 + ((m.id >> 2) % 3) as u8,
                });
            }
            MobKind::Zombie => {
                out.push(MobDrop { pos: center, block: BlockId::RawBeef, count: 1 + (m.id % 2) as u8 });
                if m.id % 6 == 0 {
                    out.push(MobDrop {
                        pos: center + Vec3::new(0.05, 0.1, 0.05),
                        block: BlockId::IronIngot,
                        count: 1,
                    });
                }
            }
            MobKind::Skeleton => {
                out.push(MobDrop { pos: center, block: BlockId::Bone, count: 1 + (m.id % 3) as u8 });
                let arrows = ((m.id >> 1) % 3) as u8;
                if arrows > 0 {
                    out.push(MobDrop {
                        pos: center + Vec3::new(0.05, 0.1, 0.05),
                        block: BlockId::Arrow,
                        count: arrows,
                    });
                }
            }
            MobKind::Reaper => {
                if m.fuse < 1.45 {
                    out.push(MobDrop { pos: center, block: BlockId::Gunpowder, count: 1 + (m.id % 2) as u8 });
                }
            }
            MobKind::Spider => {
                out.push(MobDrop { pos: center, block: BlockId::StringItem, count: 1 + (m.id % 2) as u8 });
                if m.id % 3 == 0 {
                    out.push(MobDrop {
                        pos: center + Vec3::new(0.05, 0.1, 0.05),
                        block: BlockId::SpiderEye,
                        count: 1,
                    });
                }
            }
            MobKind::Dolphin => {
                if m.id % 2 == 0 {
                    out.push(MobDrop { pos: center, block: BlockId::RawCod, count: 1 });
                }
            }
            MobKind::Cod => {
                out.push(MobDrop { pos: center, block: BlockId::RawCod, count: 1 });
                if m.id % 5 == 0 {
                    out.push(MobDrop {
                        pos: center + Vec3::new(0.05, 0.05, 0.05),
                        block: BlockId::Bone,
                        count: 1,
                    });
                }
            }
            MobKind::Salmon => {
                out.push(MobDrop { pos: center, block: BlockId::RawSalmon, count: 1 });
                if m.id % 5 == 0 {
                    out.push(MobDrop {
                        pos: center + Vec3::new(0.05, 0.05, 0.05),
                        block: BlockId::Bone,
                        count: 1,
                    });
                }
            }
            MobKind::Pufferfish => {
                out.push(MobDrop { pos: center, block: BlockId::Pufferfish, count: 1 });
            }
            MobKind::TropicalFish => {
                out.push(MobDrop { pos: center, block: BlockId::TropicalFish, count: 1 });
            }
            MobKind::Villager => {
                if m.id % 3 == 0 {
                    out.push(MobDrop { pos: center, block: BlockId::Emerald, count: 1 });
                }
            }
            MobKind::IronGolem => {
                out.push(MobDrop { pos: center, block: BlockId::IronIngot, count: 3 + (m.id % 3) as u8 });
                if m.id % 2 == 0 {
                    out.push(MobDrop {
                        pos: center + Vec3::new(0.05, 0.1, 0.05),
                        block: BlockId::RedWool,
                        count: 1,
                    });
                }
            }
        }
    }

    for (drop_block, drop_count) in crate::engine::modding::get_custom_mob_drops(m.kind.name()) {
        out.push(MobDrop {
            pos: center + Vec3::new(0.0, 0.2, 0.0),
            block: drop_block,
            count: drop_count,
        });
    }
}
