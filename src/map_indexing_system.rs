use specs::prelude::*;
use super::{ BlocksTile, Map, Position };

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (WriteExpect<'a, Map>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, BlocksTile>,
                       Entities<'a>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, pos, block, entity) = data;

        map.populate_blocked();
        for (entity, pos) in (&entity, &pos).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            // If they block, update the blocking list
            let _p : Option<&BlocksTile> = block.get(entity);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // Push the entity to the appropriate index slot. It's a Copy type,
            // so we don't need to clone it (we want to avoid moving it out of
            // the ECS)
            map.tile_content[idx].push(entity);
        }
    }
}
