use amethyst::core::bundle::{ECSBundle, Result};
use amethyst::ecs::DispatcherBuilder;
use amethyst::prelude::World;
use super::ServoUiSystem;

pub struct ServoUiBundle;
impl<'a, 'b> ECSBundle<'a, 'b> for ServoUiBundle {
    fn build(
        self,
        world: &mut World,
        dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        Ok(dispatcher.add_thread_local(ServoUiSystem::new(world)))
    }
}
