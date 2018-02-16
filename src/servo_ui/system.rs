use amethyst::ecs::{RunningTime, System};
use std::rc::Rc;
use std::sync::Arc;
use amethyst::prelude::World;
use super::window::ServoWindow;
use self::glutin::GlWindow;
use amethyst::renderer::ScreenDimensions;
use amethyst::winit::EventsLoopProxy;
extern crate glutin;

pub struct ServoUiSystem {
    window: ServoWindow,
}

impl ServoUiSystem {
    pub fn new(world: &mut World) -> Self {
        let win = world.read_resource::<Arc<GlWindow>>();
        let dimensions = world.read_resource::<ScreenDimensions>();
        let waker = world.read_resource::<Arc<EventsLoopProxy>>();
        Self {
            window: ServoWindow::new(
                waker.clone(),
                win.clone(),
                (dimensions.width(), dimensions.height()),
            ),
        }
    }
}

impl<'a> System<'a> for ServoUiSystem {
    type SystemData = ();
    fn running_time(&self) -> RunningTime {
        RunningTime::Average
    }

    fn run(&mut self, data: Self::SystemData) {}
}
