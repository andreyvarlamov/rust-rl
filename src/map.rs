use rltk::{ Algorithm2D, BaseMap, Point, RandomNumberGenerator, RGB, Rltk };
use super::{ Player, Rect, Viewshed };
use std::cmp::{ max, min };
use specs::prelude::*;

/* More derived features (from Rust itself)
   - Clone - adds .clone() method to type, allowing a copy to be made
     programmatically
   - Copy - changes the default from moving the object on assignment to making
     a copy so tile1 = tile2 leaves both values valid and not in
     a "moved from" state
   - PartialEq - allows to use == to see if two tile types match 
     Otherwise tile_type == TileType::Wall wouldn't compile */
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(
        &mut self,
        x1: i32,
        x2: i32,
        y: i32
    ) {
        for x in min(x1, x2) ..= max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(
        &mut self,
        y1: i32,
        y2: i32,
        x: i32
    ) {
        for y in min (y1, y2) ..= max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    /// Test map with solid boundaries and 400 randomly placed walls
    pub fn new_map_test() -> Map {
        /*  vec! - procedural macro 
                 - allows to define a Vec with the same syntax as an array */
        let mut map = Map{
            tiles : vec![TileType::Wall; 80*50],
            rooms : Vec::new(),
            width : 80,
            height : 50
        };

        // Make the boundary walls
        for x in 0..80 {
            let idx1 = map.xy_idx(x, 0);
            let idx2 = map.xy_idx(x, 49);
            map.tiles[idx1] = TileType::Wall;
            map.tiles[idx2] = TileType::Wall;
        }
        for y in 0..50 {
            let idx1 = map.xy_idx(0, y);
            let idx2 = map.xy_idx(79, y);
            map.tiles[idx1] = TileType::Wall;
            map.tiles[idx2] = TileType::Wall;
        }

        // Add random walls
        let mut rng = RandomNumberGenerator::new();

        for _i in 0..400 {
            let x = rng.roll_dice(1, 79);
            let y = rng.roll_dice(1, 49);
            let idx = map.xy_idx(x, y);
            if idx != map.xy_idx(40, 25) {
                map.tiles[idx] = TileType::Wall;
            }
        }

        map
    }

    /// Map with a number of rooms connected with corridors
    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map{
            tiles : vec![TileType::Wall; 80*50],
            rooms : Vec::new(),
            width : 80,
            height : 50
        };
        
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;

            let new_room = Rect::new(x, y, w, h);

            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }

            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1]
                        .center();
                    if rng.range(0, 2) == 1 {
                        /*
                          Y
                          |   p * * *
                          |         *
                          |         *
                          |         n
                          ---------------X
                        */
                        map.apply_horizontal_tunnel(
                            prev_x,
                            new_x,
                            prev_y
                        );
                        map.apply_vertical_tunnel(
                            prev_y,
                            new_y,
                            new_x
                        );
                    } else {
                        /*
                          Y
                          |   p
                          |   *
                          |   *
                          |   * * * n
                          ---------------X
                        */
                        map.apply_horizontal_tunnel(
                            prev_x,
                            new_x,
                            new_y
                        );
                        map.apply_vertical_tunnel(
                            prev_y,
                            new_y,
                            prev_x
                        );
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx : usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (_player, viewshed) in (&mut players, &mut viewsheds).join() {
        let mut y = 0;
        let mut x = 0;

        for tile in map.tiles.iter() {
            let pt = Point::new(x, y);
            if viewshed.visible_tiles.contains(&pt) {
                match tile {
                    TileType::Floor => {
                        ctx.set(
                            x,
                            y,
                            RGB::from_f32(0.5, 0.5, 0.5),
                            RGB::from_f32(0., 0., 0.),
                            rltk::to_cp437('.')
                        );
                    }
                    TileType::Wall => {
                        ctx.set(
                            x,
                            y,
                            RGB::from_f32(0.0, 1.0, 0.0),
                            RGB::from_f32(0., 0., 0.),
                            rltk::to_cp437('#')
                        );
                    }
                }
            }

            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }
}
