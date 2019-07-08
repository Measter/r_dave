use std::{
    error::Error,
};

use piston::{
    window::WindowSettings,
    event_loop::*,
    input::*,
};
use piston_window::{
    PistonWindow as Window, OpenGL
};

mod tileset;
mod level;
mod renderer;
mod game;
mod assets;
mod input;

use crate::{
    game::*,
    assets::*,
    input::Input,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const GL_VERSION: OpenGL = OpenGL::V4_5;
const SCALE: u32 = 3;
const TILE_SIZE: u32 = 16;

fn main() -> Result<()> {
    let mut window: Window = WindowSettings::new(
            "Dangerous Dave",
            [320 * SCALE, 200 * SCALE]
        )
        .graphics_api(GL_VERSION)
        .exit_on_esc(false)
        .build()?;

    // Making sure to limit the frame time and update rate.
    window.set_max_fps(30);
    window.set_ups(30);

    let mut assets = Assets::init(window.create_texture_context())?;
    let mut input = Input::default();
    let mut game = Game::init()?;

    while let Some(e) = window.next() {
        if let Some(key) = e.button_args() {
            input.update(key);
        }

        if let Some(_) = e.update_args() {
            game.input(&input);
            game.update(&mut assets);
        }

        renderer::render(&mut window, &e, &game, &assets);
    }

    Ok(())
}