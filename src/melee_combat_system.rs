use specs::prelude::*;
use super::{
    CombatStats,
    DefenseBonus,
    Equipped,
    GameLog,
    MeleePowerBonus,
    Name,
    SufferDamage,
    WantsToMelee
};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (Entities<'a>,
                       WriteStorage<'a, WantsToMelee>,
                       ReadStorage<'a, Name>,
                       ReadStorage<'a, CombatStats>,
                       WriteStorage<'a, SufferDamage>,
                       WriteExpect<'a, GameLog>,
                       ReadStorage<'a, MeleePowerBonus>,
                       ReadStorage<'a, DefenseBonus>,
                       ReadStorage<'a, Equipped>);

    fn run(&mut self, data : Self::SystemData) {
        let (
            entities,
            mut wants_to_melee,
            names,
            combat_stats,
            mut suffer_damage,
            mut game_log,
            melee_power_bonuses,
            defense_bonuses,
            equipped
        ) = data;

        for (
            entity,
            wants_to_melee,
            name,
            stats,
        ) in (
            &entities,
            &wants_to_melee,
            &names,
            &combat_stats
        ).join() {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_to_melee.target).unwrap();
                if target_stats.hp > 0 {
                    // Get attacker's offensive bonus
                    let mut offensive_bonus = 0;
                    for (
                        _item_entity, power_bonus, equipped_by
                    ) in (
                        &entities, &melee_power_bonuses, &equipped
                    ).join() {
                        if equipped_by.owner == entity {
                            offensive_bonus += power_bonus.power;
                        }
                    }

                    // Get target's defensive bonus
                    let mut defensive_bonus = 0;
                    for (
                        _item_entity, defense_bonus, equipped_by
                    ) in (
                        &entities, &defense_bonuses, &equipped
                    ).join() {
                        if equipped_by.owner == wants_to_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }

                    let damage = i32::max(
                        0,
                        (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus)
                    );

                    let target_name = names.get(wants_to_melee.target).unwrap();

                    if damage == 0 {
                        game_log.entries.push(
                            format!(
                                "{} is unable to hurt {}",
                                &name.name,
                                &target_name.name
                            )
                        );
                    } else {
                        game_log.entries.push(
                            format!(
                                "{} hits {} for {} hp.",
                                &name.name,
                                &target_name.name,
                                damage
                            )
                        );
                        SufferDamage::new_damage(
                            &mut suffer_damage,
                            wants_to_melee.target,
                            damage
                        );
                    }
                }
            }
        }

        wants_to_melee.clear();
    }
}
