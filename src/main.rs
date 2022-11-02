use inventory_system::{InventoryCollectionSystem, ItemUseSystem, ItemDropSystem};
use map_indexing_system::MapIndexingSystem;
use rltk::{GameState, Rltk, Point};
use specs::prelude::*;

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
pub use monster_ai_system::*;
mod map_indexing_system;
pub use map_indexing_system::*;
mod damage_system;
pub use damage_system::*;
mod melee_combat_system;
pub use melee_combat_system::*;
mod gui;
mod gamelog;
mod spawner;
mod inventory_system;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn, ShowInventory, ShowDropItem, ShowTargetting { range: i32, item: Entity}}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        // visibility system
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);

        // Monster AI
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);

        // Map index
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);

        // Damage systems
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);

        // inventory
        let mut pickup = InventoryCollectionSystem{};
        pickup.run_now(&self.ecs);

        // consume system
        let mut items = ItemUseSystem{};
        items.run_now(&self.ecs);

        // drop system
        let mut dropper = ItemDropSystem{};
        dropper.run_now(&self.ecs);

        DamageSystem::delete_the_dead(&mut self.ecs);
        self.ecs.maintain();
    }
}


impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();

        draw_map(&self.ecs, ctx);

        {
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
        for (pos, render) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        }
        gui::draw_ui(&self.ecs, ctx);
        }
        
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx)
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged{
                            newrunstate = RunState::ShowTargetting { range: is_item_ranged.range, item: item_entity }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUse>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUse{ item: item_entity, target: None }).expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
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
                        let mut intent = self.ecs.write_storage::<WantsToDrop>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDrop{item: item_entity}).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargetting { range, item } => {
                let target = gui::ranged_target(self, ctx, range);
                match target.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUse>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUse { item: item, target: target.1 }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
        }
    
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        DamageSystem::delete_the_dead(&mut self.ecs);

    }
}



fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);
    let mut gs = State {
        ecs: World::new()
    };

    //register components
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
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToUse>();
    gs.ecs.register::<WantsToDrop>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<InflictDamage>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AreaOfEffect>();

    


    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    
    let mut _rng = rltk::RandomNumberGenerator::new();

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);


    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }
    
    // insert resources to ecs
    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {entries: vec!["Welcome to hell".to_string()]});
    

    rltk::main_loop(context, gs)
}
