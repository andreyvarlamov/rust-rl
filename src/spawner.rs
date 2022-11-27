use rltk::{ RandomNumberGenerator, RGB };
use specs::prelude::*;
use super::{
    BlocksTile,
    CombatStats,
    Monster,
    Name,
    Player,
    Position,
    Renderable,
    Viewshed
};

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

pub fn random_monster(ecs : &mut World, x : i32, y : i32, index : usize) {
    let roll : i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { orc(ecs, x, y, index) }
        _ => { goblin(ecs, x, y, index) }
    }

}

fn orc(ecs : &mut World, x : i32, y : i32, index : usize) {
    monster(ecs, x, y, index, 'o', "Orc");
}

fn goblin(ecs : &mut World, x : i32, y : i32, index : usize) {
    monster(ecs, x, y, index, 'g', "Goblin");
}

fn monster(
    ecs : &mut World,
    x : i32,
    y : i32,
    index : usize,
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
        .with(Name { name : format!("{} #{}", &name.to_string(), index) })
        .with(Viewshed {
            visible_tiles : Vec::new(),
            range : 8,
            dirty : true
        })
        .with(BlocksTile {})
        .with(CombatStats { max_hp : 16, hp : 16, defense : 1, power : 4 })
        .build();
}
