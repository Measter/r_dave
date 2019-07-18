use piston_window::*;

use crate::{
    SCALE, TILE_SIZE,
    game::*,
    assets::*,
    tileset::*,
};

#[derive(Debug)]
pub struct Renderer {
    tick: usize,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { tick: 1 }
    }

    pub fn update(&mut self) {
        self.tick += 1;
    }

    pub fn render(&self, window: &mut PistonWindow, event: &Event, game: &Game, assets: &Assets) {
        window.draw_2d(event, |c, gl, _| {
            clear([0.0, 0.0, 0.0, 1.0], gl);

            self.draw_world(c, gl, game, assets);
            self.draw_dave(c, gl, game, assets);
            self.draw_monsters(c, gl, game, assets);
            if let Some(b) = game.dave().bullet() {
                self.draw_bullet(c, gl, b, game, assets);
            }

            if let Some(b) = game.monster_bullet() {
                self.draw_bullet(c, gl, b, game, assets);
            }
        });
    }

    fn draw_world(&self, c: Context, gl: &mut G2d, game: &Game, assets: &Assets) {
        let level = assets.get_level(game.current_level());
        let tiles = level.tiles().iter()
            .enumerate()
            .map(|(i, &t)| (i / 100, i % 100 - game.view_x() as usize, t));

        for (y, x, tile) in tiles {
            let transform = c.transform.trans((x as u32 * TILE_SIZE * SCALE) as f64, (y as u32 * TILE_SIZE * SCALE) as f64);

            let tile_image = assets.get_tile(tile.get_frame(self.tick + x * TILE_SIZE as usize));

            image(tile_image, transform, gl);
        }
    }

    fn draw_dave(&self, c: Context, gl: &mut G2d, game: &Game, assets: &Assets) {
        let dave = game.dave();
        let transform = c.transform.trans(
            ((dave.pixel_position.x - (game.view_x() as i16 * TILE_SIZE as i16)) * SCALE as i16) as f64,
            (dave.pixel_position.y * SCALE as i16) as f64,
        );

        let tile_image = if !dave.is_alive() {
            TileId::TILE_MONSTER_DYING.get_frame(self.tick)
        } else if dave.is_jetpacking() {
            match dave.direction() {
                Direction::Middle => TileId::TILE_DAVE_BASIC,
                Direction::Left => TileId::TILE_DAVE_JETPACK_LEFT.get_frame(self.tick),
                Direction::Right => TileId::TILE_DAVE_JETPACK_RIGHT.get_frame(self.tick),
            }
        } else if dave.is_on_ground() {
            match dave.direction() {
                Direction::Middle => TileId::TILE_DAVE_BASIC,
                Direction::Left => TileId::TILE_DAVE_LEFT.get_frame(dave.animation_tick),
                Direction::Right => TileId::TILE_DAVE_RIGHT.get_frame(dave.animation_tick),
            }
        } else {
            match dave.direction() {
                Direction::Middle => TileId::TILE_DAVE_BASIC,
                Direction::Left => TileId::TILE_DAVE_JUMP_LEFT,
                Direction::Right => TileId::TILE_DAVE_JUMP_RIGHT,
            }
        };

        let tile_image = assets.get_tile(tile_image);
        image(tile_image, transform, gl);
    }

    fn draw_monsters(&self, c: Context, gl: &mut G2d, game: &Game, assets: &Assets) {
        for monster in game.monsters().iter().filter(|m| m.is_not_dead()) {
            let transform = c.transform.trans(
                ((monster.pixel_position().x - (game.view_x() as i16 * TILE_SIZE as i16)) * SCALE as i16) as f64,
                (monster.pixel_position().y * SCALE as i16) as f64,
            );

            let tile_image = assets.get_tile(monster.tile_id().get_frame(self.tick));
            image(tile_image, transform, gl);
        }
    }

    fn draw_bullet(&self, c: Context, gl: &mut G2d, bullet: &Bullet, game: &Game, assets: &Assets) {
        let transform = c.transform.trans(
            ((bullet.position.x - (game.view_x() as i16 * TILE_SIZE as i16)) * SCALE as i16) as f64,
            (bullet.position.y * SCALE as i16) as f64,
        );

        let tile_image = match (bullet.source, bullet.direction) {
            (BulletSource::Monster, Direction::Left) => TileId::TILE_ENEMY_BULLET_LEFT,
            (BulletSource::Monster, _) => TileId::TILE_ENEMY_BULLET_RIGHT,
            (BulletSource::Dave, Direction::Left) => TileId::TILE_BULLET_LEFT,
            (BulletSource::Dave, _) => TileId::TILE_BULLET_RIGHT,
        };

        let tile_image = assets.get_tile(tile_image.get_frame(self.tick));
        image(tile_image, transform, gl);
    }
}