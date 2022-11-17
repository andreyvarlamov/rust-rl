use rltk::{GameState, Rltk, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};
use specs_derive::Component;

// ------------------- Components --------------------------------

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
struct Position {
    x: i32,
    y: i32
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB
}

#[derive(Component,Debug)]
// Component with no data - "tag component"
struct Player {}

// ------------------- Systems --------------------------------

// ------------------- Functions --------------------------------
fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {},
        Some(key) => match key {
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),
            _ => {},
        },
    }
}

// ------------------- Main --------------------------------
// Struct State - a class
struct State {
    ecs: World
}
// State struct implements a trait (i.e. an interface) and overrides the tick function
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.run_systems();
        player_input(self, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
impl State {
    fn run_systems(&mut self) {

        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State{
        ecs: World::new()
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    // Builder pattern - common in Rust
    // Each function returns a copy of itself (EntityByilder)
    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();

    rltk::main_loop(context, gs)
}
