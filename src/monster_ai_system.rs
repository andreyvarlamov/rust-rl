use specs::prelude::*;
use super::{ Map, Monster, Name, Position, Viewshed };
use rltk::{ a_star_search, console, DistanceAlg, Point };

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (WriteExpect<'a, Map>,
                       ReadExpect<'a, Point>,
                       WriteStorage<'a, Viewshed>,
                       WriteStorage<'a, Monster>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, mut viewshed, mut monster, name, mut pos) = data;

        for (
            mut viewshed,
            monster,
            name,
            mut pos
        ) in (
            &mut viewshed,
            &mut monster,
            &name,
            &mut pos
        ).join() {
            let distance = DistanceAlg::Pythagoras.distance2d(
                Point::new(pos.x, pos.y),
                *player_pos
            );

            if distance < 1.5 {
                // Attack goes here
                console::log(&format!("{} shouts insults", name.name));
                return;
            }

            let target_x : i32;
            let target_y : i32;

            if viewshed.visible_tiles.contains(&*player_pos) {
                target_x = player_pos.x;
                target_y = player_pos.y;
                monster.memory = (target_x, target_y);
            }
            else {
                (target_x, target_y) = monster.memory;
            }

            if target_x >= 0 && target_y >= 0 {
                let path = a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(target_x, target_y) as i32,
                    &mut *map
                );

                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
