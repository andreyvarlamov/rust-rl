use rltk::{ RandomNumberGenerator, RGB };
use specs::prelude::*;
use specs::saveload::{ MarkedBuilder, SimpleMarker };
use std::collections::HashMap;
use super::{
    AreaOfEffect,
    BlocksTile,
    CombatStats,
    Confusion,
    Consumable,
    InflictsDamage,
    Item,
    map::MAPWIDTH,
    Monster,
    Name,
    Player,
    Position,
    ProvidesHealing,
    RandomTable,
    Ranged,
    Rect,
    Renderable,
    SerializeMe,
    Viewshed
};

const MAX_SPAWNS : i32 = 4;

pub fn player(ecs : &mut World, x : i32, y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph : rltk::to_cp437('@'),
            fg : RGB::named(rltk::YELLOW),
            bg : RGB::named(rltk::BLACK),
            render_order : 0
        })
        .with(Player{})
        .with(Name { name : "Player".to_string() })
        .with(Viewshed {
            visible_tiles : Vec::new(),
            range : 8,
            dirty : true
        })
        .with(CombatStats { max_hp : 30, hp : 30, defense : 2, power : 5 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
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
            render_order : 1
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn orc(ecs : &mut World, x : i32, y : i32) {
    monster(ecs, x, y, 'o', "Orc");
}

fn goblin(ecs : &mut World, x : i32, y : i32) {
    monster(ecs, x, y, 'g', "Goblin");
}

fn health_potion(ecs : &mut World, x : i32, y : i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph : rltk::to_cp437('¡'),
            fg : RGB::named(rltk::MAGENTA),
            bg : RGB::named(rltk::BLACK),
            render_order : 2
        })
        .with(Name{ name : "Health Potion".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(ProvidesHealing{ heal_amount : 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs : &mut World, x : i32, y : i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph : rltk::to_cp437(')'),
            fg : RGB::named(rltk::CYAN),
            bg : RGB::named(rltk::BLACK),
            render_order : 2
        })
        .with(Name{ name : "Magic Missile Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range : 6 })
        .with(InflictsDamage{ damage : 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs : &mut World, x : i32, y : i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph : rltk::to_cp437(')'),
            fg : RGB::named(rltk::ORANGE),
            bg : RGB::named(rltk::BLACK),
            render_order : 2
        })
        .with(Name{ name : "Fireball Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range : 6 })
        .with(InflictsDamage{ damage : 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs : &mut World, x : i32, y : i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph : rltk::to_cp437(')'),
            fg : RGB::named(rltk::PINK),
            bg : RGB::named(rltk::BLACK),
            render_order : 2
        })
        .with(Name{ name : "Confusion Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range : 6})
        .with(Confusion{ turns: 4})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn room_table() -> RandomTable {
    return RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2)
        .add("Confusion Scroll", 2)
        .add("Magic Missile Scroll", 4);
}

/// Fill room with stuff
#[allow(clippy::map_entry)]
pub fn spawn_room(ecs : &mut World, room : &Rect) {
    let spawn_table = room_table();
    let mut spawn_points : HashMap<usize, String> = HashMap::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_SPAWNS + 3) - 3;

        for _i in 0 .. num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let delta_x = rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let delta_y = rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let x = (room.x1 + delta_x) as usize;
                let y = (room.y1 + delta_y) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            "Goblin" => goblin(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Confusion Scroll" => confusion_scroll(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            _ => {}
        }
    }
}
