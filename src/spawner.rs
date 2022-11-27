use rltk::{ RandomNumberGenerator, RGB };
use specs::prelude::*;
use super::{
    BlocksTile,
    CombatStats,
    map::MAPWIDTH,
    Monster,
    Name,
    Player,
    Position,
    Rect,
    Renderable,
    Viewshed
};

const MAX_MONSTERS : i32 = 4;
const MAX_ITEMS : i32 = 2;

pub fn player(ecs : &mut World, x : i32, y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph : rltk::to_cp437('@'),
            fg : RGB::named(rltk::BLUE),
            bg : RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Name { name : "Player".to_string() })
        .with(Viewshed {
            visible_tiles : Vec::new(),
            range : 8,
            dirty : true
        })
        .with(CombatStats { max_hp : 30, hp : 30, defense : 2, power : 5 })
        .build()
}

pub fn random_monster(ecs : &mut World, x : i32, y : i32) {
    let roll : i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { orc(ecs, x, y) }
        _ => { goblin(ecs, x, y) }
    }

}

fn orc(ecs : &mut World, x : i32, y : i32) {
    monster(ecs, x, y, 'o', "Orc");
}

fn goblin(ecs : &mut World, x : i32, y : i32) {
    monster(ecs, x, y, 'g', "Goblin");
}

fn monster(
    ecs : &mut World,
    x : i32,
    y : i32,
    glyph_char : char,
    name : &str
) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph : rltk::to_cp437(glyph_char),
            fg : RGB::named(rltk::RED),
            bg : RGB::named(rltk::BLACK),
        })
        .with(Monster {})
        .with(Name { name : format!("{}", &name.to_string()) })
        .with(Viewshed {
            visible_tiles : Vec::new(),
            range : 8,
            dirty : true
        })
        .with(BlocksTile {})
        .with(CombatStats { max_hp : 16, hp : 16, defense : 1, power : 4 })
        .build();
}

/// Fill room with stuff
pub fn spawn_room(ecs : &mut World, room : &Rect) {
    let mut monster_spawn_points : Vec<usize> = Vec::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;

        for _i in 0 .. num_monsters {
            let mut added = false;
            while !added {
                let delta_x = rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let delta_y = rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let x = (room.x1 + delta_x) as usize;
                let y = (room.y1 + delta_y) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }
}
