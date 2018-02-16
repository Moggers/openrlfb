use amethyst::prelude::*;
use amethyst::Result;
use amethyst::renderer::{DisplayConfig, DrawShaded, Event, Pipeline, PosNormTex, RenderBundle,
                         Stage};
use boilerplate;
use servo_ui::ServoUiBundle;

pub struct GameState;

impl State for GameState {
    fn on_start(&mut self, world: &mut World) {
        boilerplate::initialise_camera(world);
    }
    fn handle_event(&mut self, _: &mut World, _: Event) -> Trans {
        Trans::None
    }
}

pub fn run() -> Result<()> {
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.3, 0.3, 0.4, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );

    let path = "./resources/display_config.ron";
    let config = DisplayConfig::load(&path);

    let mut world = Application::build("resources/", GameState)?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        .with_bundle(ServoUiBundle {})?
        .build()?;
    world.run();
    Ok(())
}
