use specs::prelude::*;
use specs_derive::*;
use rltk::{ RGB, Point };

/* POD (Plain Old Data) - no logic - "pure" ECS
   2 reasons to use this model:
   1) keeps all the logic code in "systems" part of ECS
   2) very fast to keep all of the positions next to each other in memory with no redirects */

// Without using specs-derive:
// struct Position {
//     x: i32,
//     y: i32
// }

// impl Component for Position {
//     type Storage = VecStorage<Self>;
// }

// Using specs-derive
// A derive macro: "from ths basic data derive the boilerplate needed for Component"
#[derive(Component)] 
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB
}

#[derive(Component,Debug)]
// Component with no data - "tag component"
pub struct Player {}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles : Vec<Point>,
    pub range : i32
}
