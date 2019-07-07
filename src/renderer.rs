use piston_window::*;

use crate::{
    SCALE,
    game::*,
    assets::*,
};

pub fn render(window: &mut PistonWindow, event: &Event, game: &Game, assets: &Assets) {
    window.draw_2d(event, |c, gl, _| {

        clear([0.0, 0.0, 0.0, 1.0], gl);

        let level = assets.get_level(game.current_level());
        let tiles = level.tiles().iter()
            .enumerate()
            .map(|(i, &t)| (i/100, i%100 - game.view_x() as usize, t));

        for (y, x, tile) in tiles {
            let transform = c.transform.trans((x as u32 * 16*SCALE) as f64, (y as u32 * 16*SCALE) as f64);

            let tile_image = assets.get_tile(tile);

            image(tile_image, transform, gl);
        }
    });
}
