//! Wykonywanie niestandardowych komend czatu zdefiniowanych w modach (`execute_mod_command`).

use crate::features::entities::MobKind;
use crate::features::player::ItemStack;
use crate::features::ui::chat::{parse_block_id, ChatState, CommandContext};
use crate::features::world::World;

use super::registry::find_custom_block_by_name;
use super::types::{ModAction, ModCommandDef};

pub fn execute_mod_command(
    cmd_def: &ModCommandDef,
    chat: &mut ChatState,
    ctx: &mut CommandContext,
) {
    for action in &cmd_def.actions {
        match action {
            ModAction::Message(msg) => {
                chat.add_info(msg);
            }
            ModAction::Give { item, count } => {
                let resolved = parse_block_id(item).or_else(|| find_custom_block_by_name(item));

                if let Some(block) = resolved {
                    let mut remaining = *count;
                    for slot in &mut ctx.player.inventory.hotbar {
                        if remaining == 0 {
                            break;
                        }
                        if let Some(st) = slot {
                            if st.block == block && st.count < 64 {
                                let add = remaining.min(64 - st.count);
                                st.count += add;
                                remaining -= add;
                            }
                        } else {
                            let add = remaining.min(64);
                            *slot = Some(ItemStack::new(block, add));
                            remaining -= add;
                        }
                    }
                    for slot in &mut ctx.player.inventory.main {
                        if remaining == 0 {
                            break;
                        }
                        if let Some(st) = slot {
                            if st.block == block && st.count < 64 {
                                let add = remaining.min(64 - st.count);
                                st.count += add;
                                remaining -= add;
                            }
                        } else {
                            let add = remaining.min(64);
                            *slot = Some(ItemStack::new(block, add));
                            remaining -= add;
                        }
                    }
                    chat.add_success(format!("Gave {} {} to player", count, block.name()));
                } else {
                    chat.add_error(format!("Unknown item in command: {}", item));
                }
            }
            ModAction::Heal => {
                ctx.player.health = 20.0;
                ctx.player.hunger = 20.0;
                ctx.player.saturation = 5.0;
                ctx.player.exhaustion = 0.0;
                chat.add_success("Player restored to full health and hunger.");
            }
            ModAction::FlyToggle => {
                *ctx.is_flying = !*ctx.is_flying;
                chat.add_info(format!(
                    "Flight mode: {}",
                    if *ctx.is_flying { "ENABLED" } else { "DISABLED" }
                ));
            }
            ModAction::Time(ticks) => {
                let frac = (ticks % 24000.0) / 24000.0;
                ctx.world.time_of_day = World::DAY_DURATION * frac;
                chat.add_success(format!("Time set to {:.0} ticks", ticks));
            }
            ModAction::SpawnMob { mob, count } => {
                let kind_opt = match mob.to_ascii_lowercase().as_str() {
                    "pig" => Some(MobKind::Pig),
                    "cow" => Some(MobKind::Cow),
                    "chicken" => Some(MobKind::Chicken),
                    "zombie" => Some(MobKind::Zombie),
                    "skeleton" => Some(MobKind::Skeleton),
                    "reaper" => Some(MobKind::Reaper),
                    "spider" => Some(MobKind::Spider),
                    _ => None,
                };
                if let Some(kind) = kind_opt {
                    let player_pos = ctx.camera.position;
                    for _ in 0..*count {
                        ctx.mob_manager.spawn_mob(kind, player_pos);
                    }
                    chat.add_success(format!("Spawned {} {}", count, kind.name()));
                } else {
                    chat.add_error(format!("Unknown mob type: {}", mob));
                }
            }
            ModAction::ClearInventory => {
                ctx.player.inventory.hotbar = [None; 10];
                ctx.player.inventory.main = [None; 27];
                ctx.player.inventory.held_item = None;
                chat.add_success("Player inventory cleared.");
            }
        }
    }
}
