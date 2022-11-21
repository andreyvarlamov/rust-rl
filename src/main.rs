use rltk::{ GameState, Rltk, RGB, TextAlign };
use specs::prelude::*;

// Crate files
mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;

// Consts
const SHOW_FPS : bool = false;

// Struct State - a class
pub struct State {
    ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

// State struct implements a trait (i.e. an interface)
// and overrides the tick function
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }

        if SHOW_FPS {
            ctx.draw_box(39, 0, 20, 3,
                         RGB::named(rltk::WHITE),
                         RGB::named(rltk::BLACK)
            );
            ctx.printer(
                58,
                1,
                &format!("#[pink]FPS: #[]{}", ctx.fps),
                TextAlign::Right,
                None,
            );
        }
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
    gs.ecs.register::<Viewshed>();

    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(map);

    // Builder pattern - common in Rust
    // Each function returns a copy of itself (EntityByilder)
    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Viewshed { visible_tiles : Vec::new(), range : 8, dirty :true })
        .build();

    rltk::main_loop(context, gs)
}
