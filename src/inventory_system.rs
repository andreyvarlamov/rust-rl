use specs::prelude::*;
use super::{
    CombatStats,
    Consumable,
    gamelog::GameLog,
    InBackpack,
    InflictsDamage,
    Map,
    Name,
    Position,
    ProvidesHealing,
    SufferDamage,
    WantsToUseItem,
    WantsToDropItem,
    WantsToPickupItem
};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (ReadExpect<'a, Entity>,
                       WriteExpect<'a, GameLog>,
                       WriteStorage<'a, WantsToPickupItem>,
                       WriteStorage<'a, Position>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, InBackpack>);

    fn run(&mut self, data : Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            mut wants_pickup,
            mut positions,
            names,
            mut backpack
        ) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item,
                            InBackpack{ owner : pickup.collected_by })
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (WriteExpect<'a, Map>,
                       ReadExpect<'a, Entity>,
                       WriteExpect<'a, GameLog>,
                       Entities<'a>,
                       WriteStorage<'a, WantsToUseItem>,
                       ReadStorage<'a, Name>,
                       ReadStorage<'a, Consumable>,
                       ReadStorage<'a, ProvidesHealing>,
                       ReadStorage<'a, InflictsDamage>,
                       WriteStorage<'a, CombatStats>,
                       WriteStorage<'a, SufferDamage>);

    fn run(&mut self, data : Self::SystemData) {
        let (
            map,
            player_entity,
            mut gamelog,
            entities,
            mut wants_use,
            names,
            consumables,
            healing,
            damaging,
            mut combat_stats,
            mut suffer_damage
        ) = data;

        for (entity, useitem, stats) in (&entities, &wants_use, &mut combat_stats).join() {
            let mut used_item = true;

            // If item heals, apply the healing
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!("You drink the {}, healing {} hp.",
                                                     names.get(useitem.item).unwrap().name,
                                                     healer.heal_amount));
                    }
                }
            }

            // If it inflicts damage, apply it to the target cell
            let item_damages = damaging.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    let target_point = useitem.target.unwrap();
                    let idx = map.xy_idx(target_point.x, target_point.y);
                    used_item = false;
                    for mob in map.tile_content[idx].iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(
                                format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name.name,
                                    mob_name.name,
                                    damage.damage
                                )
                            );
                        }

                        used_item = true;
                    }
                }
            }


            // Remove consumable after its use
            let consumable = consumables.get(useitem.item);
            match consumable {
                None => {}
                Some(_) => {
                    entities.delete(useitem.item).expect("Delete failed");
                }
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (ReadExpect<'a, Entity>,
                       WriteExpect<'a, GameLog>,
                       Entities<'a>,
                       WriteStorage<'a, WantsToDropItem>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, InBackpack>);

    fn run(&mut self, data : Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let pos = positions.get(entity).unwrap();
            positions.insert(to_drop.item, Position{ x : pos.x, y : pos.y })
                .expect("Unabled to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.",
                                             names.get(to_drop.item).unwrap().name));
            }
        }
        wants_drop.clear();
    }
}
