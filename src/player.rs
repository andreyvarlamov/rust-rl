use rltk::{ Point, Rltk, VirtualKeyCode };
use specs::prelude::*;
use super::{ 
    CombatStats,
    Map,
    Player,
    Position,
    RunState,
    State,
    Viewshed,
    WantsToMelee
};
use std::cmp::{ min, max };

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    for (
        entity,
        _player,
        pos,
        viewshed
    ) in (
        &entities,
        &players,
        &mut positions,
        &mut viewsheds
    ).join() {
        let new_x = pos.x + delta_x;
        let new_y = pos.y + delta_y;

        if new_x < 1 || new_x > map.width - 1 ||
            new_y  < 1 || new_y > map.height - 1 {
            return;
        }

        let destination_idx = map.xy_idx(new_x, new_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee.
                    insert(entity, WantsToMelee { target : *potential_target }).
                    expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, new_x));
            pos.y = min(49, max(0, new_y));

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => { return RunState::AwaitingInput},
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),

            // Diagonals
            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::M => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::N => try_move_player(-1, 1, &mut gs.ecs),

            _ => { return RunState::AwaitingInput },
        },
    }
    RunState::PlayerTurn
}
