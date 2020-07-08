use amethyst::{
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    input::{InputBundle, StringBindings, InputEvent, VirtualKeyCode},
    ecs::{Component, DenseVecStorage, System, WriteStorage, ReadStorage, Read, Join},
};
use ron::de::from_reader;
use serde::Deserialize;
use std::{path::PathBuf, fs::File, string::String, collections::HashMap};

enum SimStatus {
    Wait,
    Run
}

impl Default for SimStatus {
    fn default() -> Self {
        SimStatus::Wait
    }
}

struct Stockpile {
    goods: HashMap<String, f32>
}
struct Fills {
    fills: HashMap<String, f32>
}
struct Needs {
    needs: HashMap<String, f32>
}

impl Component for Stockpile {
    type Storage = DenseVecStorage<Self>;
}
impl Component for Fills {
    type Storage = DenseVecStorage<Self>;
}
impl Component for Needs {
    type Storage = DenseVecStorage<Self>;
}

struct ConsumptionSystem;

impl<'a> System<'a> for ConsumptionSystem {
    type SystemData = (
        WriteStorage<'a, Stockpile>,
        WriteStorage<'a, Fills>,
        ReadStorage<'a, Needs>,
        Read<'a, SimStatus>,
    );

    fn run(&mut self, (mut stockpiles, mut fills, needs, sim_status): Self::SystemData) {
        match *sim_status {
            SimStatus::Run => {
                for (stockpile, fill, need) in (&mut stockpiles, &mut fills, &needs).join() {
                    println!("Consumption Before");
        
                    for (name, amount) in stockpile.goods.iter() {
                        println!("Has {} {}", amount, name);
                    }
        
                    fill.fills.clear();
        
                    for (name, amount) in need.needs.iter() {
                        if let Some(good) = stockpile.goods.get_mut(name) {
                            if *good / 2. > *amount {
                                *good -= *amount;
        
                                fill.fills.insert(name.clone(), 1.);
                            } else {
                                *good /= 2.;
        
                                fill.fills.insert(name.clone(), *good / *amount);
                            }
                        } else {
                            fill.fills.insert(name.clone(), 0.);
                        }
                    }
        
                    println!("Consumption After");
        
                    for (name, amount) in stockpile.goods.iter() {
                        println!("Has {} {}", amount, name);
                    }
                    for (name, amount) in fill.fills.iter() {
                        println!("Filled {} {}", amount, name);
                    }
                }
            }
            _ => {}
        }
    }
}

struct LoadState {
    defines: PathBuf,
}
struct WaitState {
    iter: u64,
}
struct SimState {
    iter: u64
}

impl SimpleState for LoadState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        world.insert(SimStatus::Wait);

        let mut goods = HashMap::new();
        let mut needs = HashMap::new();

        goods.insert("Wheat".to_string(), 10.);
        goods.insert("Meat".to_string(), 5.);
        goods.insert("Water".to_string(), 20.);

        needs.insert("Wheat".to_string(), 1.);
        needs.insert("Meat".to_string(), 0.5);
        needs.insert("Water".to_string(), 1.);

        world
            .create_entity()
            .with(Stockpile { goods })
            .with(Needs { needs })
            .with(Fills { fills: HashMap::new()})
            .build();
    }

    fn update(&mut self, _: &mut StateData<GameData>) -> SimpleTrans {
        SimpleTrans::Switch(Box::new(WaitState{iter: 0}))
    }
}
impl SimpleState for WaitState {
    fn on_resume(&mut self, _: StateData<GameData>) {
        self.iter += 1;
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Input(input_event) = event {
            if let InputEvent::KeyReleased {key_code, ..} = input_event {
                if let VirtualKeyCode::Space = key_code {
                    println!("Iteration {} Beginning", self.iter);

                    return SimpleTrans::Push(Box::new(SimState{iter: self.iter}))
                }
            }
        }

        SimpleTrans::None
    }
}
impl SimpleState for SimState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let mut sim_status = data.world.write_resource::<SimStatus>();
        *sim_status = SimStatus::Run;
    }
    fn on_stop(&mut self, data: StateData<GameData>) {
        let mut sim_status = data.world.write_resource::<SimStatus>();
        *sim_status = SimStatus::Wait;
    }

    fn update(&mut self, _: &mut StateData<GameData>) -> SimpleTrans {
        println!("Iteration {} Running", self.iter);
        println!("Iteration {} Ending", self.iter);

        SimpleTrans::Pop
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let defines_dir = app_root.join("defines").join("define.ron");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");

    let input_bundle = InputBundle::<StringBindings>::new();

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        .with_bundle(input_bundle)?
        .with(ConsumptionSystem, "Consumption System", &[]);

    let load_state = LoadState {
        defines: defines_dir
    };

    let mut game = Application::new(assets_dir, load_state, game_data)?;
    game.run();

    Ok(())
}
