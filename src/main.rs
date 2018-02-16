extern crate amethyst;

mod application;
mod boilerplate;
mod servo_ui;

fn main() {
    println!("Hello, world!");
    println!("{:?}", application::run())
}
