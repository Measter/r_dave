use piston_window::*;

use crate::{SCALE, game::*, assets::*, tileset::*, TILE_SIZE};

pub fn render(window: &mut PistonWindow, event: &Event, game: &Game, assets: &Assets) {
    window.draw_2d(event, |c, gl, _| {
        clear([0.0, 0.0, 0.0, 1.0], gl);

        draw_world(c, gl, game, assets);
        draw_dave(c, gl, game, assets);
    });
}

fn draw_world(c: Context, gl: &mut G2d, game: &Game, assets: &Assets ) {
    let level = assets.get_level(game.current_level());
    let tiles = level.tiles().iter()
        .enumerate()
        .map(|(i, &t)| (i/100, i%100 - game.view_x() as usize, t));

    for (y, x, tile) in tiles {
        let transform = c.transform.trans((x as u32 * TILE_SIZE *SCALE) as f64, (y as u32 * TILE_SIZE * SCALE) as f64);

        let tile_image = assets.get_tile(tile);

        image(tile_image, transform, gl);
    }
}

fn draw_dave(c: Context, gl: &mut G2d, game: &Game, assets: &Assets) {
    let dave = game.dave();
    let transform = c.transform.trans(
        (dave.pixel_position.x * SCALE as i16) as f64,
        (dave.pixel_position.y * SCALE as i16) as f64,
    );

    let tile_image = assets.get_tile(TileId::tile_dave_basic());
    image(tile_image, transform, gl);
}
