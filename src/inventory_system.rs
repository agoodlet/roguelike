use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog, CombatStats, WantsToUse, Consumable, ProvidesHealing, WantsToDrop, InflictDamage, SufferDamage, Map, AreaOfEffect};

//TODO refactor inventory system to select items with movement keys then have hotkeys for use/drop/etc

pub struct InventoryCollectionSystem {}

impl<'a> System<'a> for InventoryCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>);

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_to_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_to_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{owner: pickup.collected_by}).expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You pick up the {}", names.get(pickup.item).unwrap().name));
            }
        }
        wants_to_pickup.clear();
    }
}


pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUse>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, ProvidesHealing>,
                        WriteStorage<'a, CombatStats>,
                        ReadStorage<'a, Consumable>,
                        ReadStorage<'a, InflictDamage>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, AreaOfEffect>);

    fn run(&mut self, data: Self::SystemData){
        let (player_entity, mut gamelog, map, entities, mut wants_to_use, names, healing, mut combat_stats, consumables, inflict_damage, mut suffer_damage, aoe) = data;

        for (entity, useitem) in (&entities, &wants_to_use).join() {            
            let mut used_item = true;
        
            //targetting
            let mut targets: Vec<Entity> = Vec::new();
            match useitem.target{
                None => { targets.push(*player_entity);}
                Some(target) => {
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => {
                            //single target in tile
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter(){ 
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                        }
                    }
                }
            }

            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("You use the {}, healing {} hp", names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }
                        }
                    }
                }
            }

            // If it inflicts damage, apply it to the target cell
            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!("You use {} on {}, inflicting {} hp.", item_name.name, mob_name.name, damage.damage));
                        }
                    
                        used_item = true;
                    }
                }
            }
        }
        wants_to_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDrop>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_to_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_to_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position { x: dropper_pos.x, y: dropper_pos.y }).expect("unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}", names.get(to_drop.item).unwrap().name));
            }
        }
        wants_to_drop.clear();
    }
}