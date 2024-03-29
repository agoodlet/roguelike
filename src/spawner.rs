use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;
use crate::AreaOfEffect;

use super::{CombatStats, Player, Renderable, Name, Position, Viewshed, Monster, BlocksTile, Rect, Item, ProvidesHealing, map::MAPWIDTH, Consumable, InflictDamage, Ranged};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

/// spawns the player at a specified location
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true  })
        .with(Name{name: "Player".to_string()})
        .with(CombatStats {max_hp: 30, hp: 30, defense: 2, power: 17})
        .build()
}

/// spawns random monsters at a specified location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1,2);
    }

    match roll {
        1 => { orc(ecs, x, y)},
        _ => { goblin(ecs, x, y)}
    }
}

pub fn orc(ecs: &mut World, x: i32 , y: i32) {monster(ecs,x,y,rltk::to_cp437('o'), "Orc");}
pub fn goblin(ecs: &mut World, x: i32 , y: i32) {monster(ecs,x,y,rltk::to_cp437('g'), "Goblin");}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
            .with(Position{x,y})
            .with(Renderable{
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
                render_order: 1
            })
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true})
            .with(Monster{})
            .with(Name{name: name.to_string()})
            .with(BlocksTile{})
            .with(CombatStats {max_hp: 16, hp: 16, defense: 1, power: 2})
            .build();
}

/// Fills a room with stuff
pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) -3;

        for _i in 0.. num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }
        
        for _i in 0 .. num_items {
            let mut added = false; 
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
    
        }
    }

    //spawn monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }

    for idx in item_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_item(ecs, x as i32, y as i32)
    }

}

fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 3);
    }

    match roll {
        1 => { health_potion(ecs, x, y)}
        2 => { magic_missile(ecs, x, y)}
        _ => { fireball(ecs, x, y)}
    }
}

pub fn health_potion(ecs: &mut World, x: i32, y: i32) { 
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437('¡'),
        fg: RGB::named(rltk::MAGENTA),
        bg: RGB::named(rltk::BLACK),
        render_order: 2
    })
    .with(Name {name: "Health Potion".to_string()})
    .with(Item{})
    .with(Consumable{})
    .with(ProvidesHealing{heal_amount: 8})
    .build();
}

pub fn magic_missile(ecs: &mut World, x:i32, y:i32) {
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437(')'),
        fg: RGB::named(rltk::CYAN),
        bg: RGB::named(rltk::BLACK),
        render_order: 2
    })
    .with(Name {name: "Scroll of Magic Missile".to_string()})
    .with(Item{})
    .with(Consumable{})
    .with(Ranged{ range: 6})
    .with(InflictDamage{ damage: 17})
    .build();    
}

pub fn fireball(ecs: &mut World, x:i32, y:i32) {
    ecs.create_entity()
    .with(Position {x, y})
    .with(Renderable {
        glyph: rltk::to_cp437(')'),
        fg: RGB::named(rltk::ORANGE),
        bg: RGB::named(rltk::BLACK),
        render_order: 2
    })
    .with(Name {name: "Scroll of Fireball".to_string()})
    .with(Item{})
    .with(Consumable{})
    .with(Ranged{ range: 6})
    .with(InflictDamage{ damage: 20})
    .with(AreaOfEffect{radius: 3})
    .build();    
}