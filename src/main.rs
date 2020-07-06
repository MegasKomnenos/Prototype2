use amethyst::{
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    input::{InputBundle, StringBindings, InputEvent, VirtualKeyCode},
};

#[derive(Default)]
struct WaitState {
    iter: u64,
}

impl SimpleState for WaitState {
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

struct SimState {
    iter: u64
}

impl SimpleState for SimState {
    fn update(&mut self, _: &mut StateData<GameData>) -> SimpleTrans {
        println!("Iteration {} Running", self.iter);
        println!("Iteration {} Ending", self.iter);
        self.iter += 1;

        SimpleTrans::Push(Box::new(WaitState{iter: self.iter}))
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
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
        .with_bundle(input_bundle)?;

    let mut game = Application::new(assets_dir, WaitState::default(), game_data)?;
    game.run();

    Ok(())
}
