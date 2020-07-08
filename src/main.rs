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

enum BeliefEntry {
    Good {
        id: u8,
        id_other: u8,
        trust: f32,
        amount: f32,
    },
}

impl PartialEq for BeliefEntry {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Good { id, id_other, ..} => match other {
                Self::Good { id: other_id, id_other: other_id_other, ..} => (id == other_id && id_other == other_id_other) || (id == other_id_other && id_other == other_id),
                _ => false,
            },
        }
    }
}
impl Eq for BeliefEntry {}

enum Activity {
    Production {
        id: u8,
        cost_fixed: f32,
        cost_scale: f32,
        inputs: Vec<f32>,
        outputs: Vec<f32>,
    },
    Trade {
        cost_fixed: f32,
        cost_scale: f32,
    },
}

struct Stockpile {
    goods: Vec<f32>
}
struct Fills {
    fills: Vec<f32>
}
struct Needs {
    needs: Vec<f32>
}
struct Keeps {
    keeps: Vec<f32>
}
struct Beliefs {
    beliefs: Vec<BeliefEntry>
}
struct Acts {
    acts: Vec<Activity>
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
impl Component for Keeps {
    type Storage = DenseVecStorage<Self>;
}
impl Component for Beliefs {
    type Storage = DenseVecStorage<Self>;
}
impl Component for Acts {
    type Storage = DenseVecStorage<Self>;
}

struct DecaySystem;
struct ConsumptionSystem;
struct ActivitySystem;

impl<'a> System<'a> for DecaySystem {
    type SystemData = (
        WriteStorage<'a, Keeps>,
        Read<'a, SimStatus>,
    );

    fn run(&mut self, (mut keeps, sim_status): Self::SystemData) {
        match *sim_status {
            SimStatus::Run => {
                for keep in (&mut keeps).join() {
                    for amount in keep.keeps.iter_mut() {
                        *amount *= 0.5;
                    }
                }
            }
            _ => {}
        }
    }
}
impl<'a> System<'a> for ConsumptionSystem {
    type SystemData = (
        WriteStorage<'a, Stockpile>,
        WriteStorage<'a, Fills>,
        WriteStorage<'a, Keeps>,
        ReadStorage<'a, Needs>,
        Read<'a, SimStatus>,
    );

    fn run(&mut self, (mut stockpiles, mut fills, mut keeps, needs, sim_status): Self::SystemData) {
        match *sim_status {
            SimStatus::Run => {
                for (stockpile, fill, keep, need) in (&mut stockpiles, &mut fills, &mut keeps, &needs).join() {
                    for (id, amount) in need.needs.iter().enumerate() {
                        if stockpile.goods[id] / 2. > *amount {
                            stockpile.goods[id] -= *amount;
                            keep.keeps[id] += *amount;
                            fill.fills[id] = 1.;
                        } else {
                            stockpile.goods[id] /= 2.;
                            keep.keeps[id] += stockpile.goods[id];
                            fill.fills[id] = stockpile.goods[id] / *amount;
                        }
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

        world
            .create_entity()
            .with(Stockpile { goods: vec![10., 5., 20.] })
            .with(Needs { needs: vec![1., 0.5, 1.] })
            .with(Fills { fills: vec![0., 0., 0.] })
            .with(Keeps { keeps: vec![0., 0., 0.] })
            .build();
    }

    fn update(&mut self, _: &mut StateData<GameData>) -> SimpleTrans {
        SimpleTrans::Switch(Box::new(WaitState{iter: 0}))
    }
}
impl SimpleState for WaitState {
    fn on_resume(&mut self, data: StateData<GameData>) {
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
        .with(DecaySystem, "Decay System", &[])
        .with(ConsumptionSystem, "Consumption System", &["Decay System"]);

    let load_state = LoadState {
        defines: defines_dir
    };

    let mut game = Application::new(assets_dir, load_state, game_data)?;
    game.run();

    Ok(())
}
