use rltk::{ GameState, Point, Rltk, RGB, TextAlign };
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
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::*;
mod gui;
mod gamelog;
use gamelog::GameLog;
mod spawner;
mod inventory_system;
use inventory_system::*;

// Consts
const SHOW_FPS : bool = false;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem
}

// Struct State - a class
pub struct State {
    pub ecs : World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut map_index = MapIndexingSystem{};
        map_index.run_now(&self.ecs);
        let mut melee_combat = MeleeCombatSystem{};
        melee_combat.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut potions = PotionUseSystem{};
        potions.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

// State struct implements a trait (i.e. an interface)
// and overrides the tick function
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDrinkPotion>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(),
                            WantsToDrinkPotion{ potion : item_entity }
                        ).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(
                            *self.ecs.fetch::<Entity>(),
                            WantsToDropItem{ item : item_entity }
                        ).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        for (pos, render) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        gui::draw_ui(&self.ecs, ctx);

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
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .with_tile_dimensions(20, 20)
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State{
        ecs : World::new()
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDrinkPotion>();
    gs.ecs.register::<WantsToDropItem>();

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog { entries : vec!["Hello".to_string()] });

    let map : Map = Map::new_map_rooms_and_corridors();

    let (player_x, player_y) = map.rooms[0].center();

    // Builder pattern - common in Rust
    // Each function returns a copy of itself (EntityByilder)
//    let player_entity = gs.ecs
//        .create_entity()
//        .with(Position { x : player_x, y : player_y })
//        .with(Renderable {
//            glyph : rltk::to_cp437('@'),
//            fg : RGB::named(rltk::BLUE),
//            bg : RGB::named(rltk::BLACK),
//        })
//        .with(Player{})
//        .with(Name { name : "Player".to_string() })
//        .with(Viewshed {
//            visible_tiles : Vec::new(),
//            range : 8,
//            dirty : true
//        })
//        .with(CombatStats { max_hp : 30, hp : 30, defense : 2, power : 5 })
//        .build();

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(Point::new(player_x, player_y));

    rltk::main_loop(context, gs)
}
