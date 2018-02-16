use amethyst::ecs::World;
use amethyst::renderer::{Camera, Projection};
use amethyst::core::cgmath::Deg;
use amethyst::core::transform::GlobalTransform;

/// This function initialises a camera and adds it to the world.
pub fn initialise_camera(world: &mut World) {
    use amethyst::core::cgmath::Matrix4;
    let transform =
        Matrix4::from_translation([0.0, 0.0, -4.0].into()) * Matrix4::from_angle_y(Deg(180.));
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(GlobalTransform(transform.into()))
        .build();
}
