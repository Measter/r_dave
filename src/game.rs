use std::ops::Add;

use crate::{
    level::*, Result, TILE_SIZE,
    assets::Assets,
    input::Input,
    tileset::*,
    dave::*,
};

pub struct Game {
    dave: Dave,
    misc: MiscParts,
}

#[derive(Debug)]
struct MiscParts {
    level: LevelId,
    view_x: u8,
    scroll_x: i16,
    score: u32,
    lives: u8,
    has_trophy: bool,
    quit: bool,
}

impl Game {
    pub fn current_level(&self) -> LevelId {
        self.misc.level
    }

    pub fn view_x(&self) -> u8 {
        self.misc.view_x
    }

    pub fn dave(&self) -> &Dave {
        &self.dave
    }

    pub fn quit(&self) -> bool {
        self.misc.quit
    }
}

impl Game {
    pub fn init() -> Result<Self> {
        let mut game = Game {
            misc: MiscParts {
                level: LevelId::first_level().next().unwrap().next().unwrap(),
                view_x: 0,
                scroll_x: 0,
                score: 0,
                lives: 3,
                has_trophy: false,
                quit: false,
            },

            dave: Dave::init(),
        };

        game.start_level();

        Ok(game)
    }

    fn scroll_screen(&mut self) {
        match self.dave.position.x - self.misc.view_x {
            18..=255 => self.misc.scroll_x = 15,
            0..=1    => self.misc.scroll_x = -15,
            _ => {}
        }

        if self.misc.scroll_x > 0 {
            self.misc.view_x = (self.misc.view_x + 1).min(80);
            self.misc.scroll_x -= 1;
        }

        if self.misc.scroll_x < 0 {
            self.misc.view_x = self.misc.view_x.checked_sub(1).unwrap_or(0);
            self.misc.scroll_x += 1;
        }
    }

    fn pickup_item(&mut self, assets: &mut Assets) {
        if self.dave.check_pickup.x == 0 && self.dave.check_pickup.y == 0 {
            return;
        }

        let level = assets.get_level_mut(self.misc.level);
        let tile_type = &mut level.tiles_mut()[self.dave.check_pickup.y as usize * 100 + self.dave.check_pickup.x as usize];

        match *tile_type {
            // Add score and special item cases here later.
            TileId::TILE_JETPACK => self.dave.has_jetpack = true,
            TileId::TILE_GUN => self.dave.has_gun = true,
            t if t.is_trophy() => {
                self.misc.score += 1000;
                self.misc.has_trophy = true;
            },
            _ => {}
        }

        *tile_type = TileId::TILE_BLANK;
        self.dave.check_pickup = Default::default();
    }

    fn check_dave_collision(&mut self, assets: &Assets) {
        let offsets = [
            (4,  -1),
            (10, -1),
            (12,  4),
            (12,  12),
            (10,  16),
            (4,   16),
            (3,   12),
            (3,   4),
        ];

        for (i, &offset) in offsets.iter().enumerate() {
            let coord = self.dave.pixel_position + offset;
            let col_type = is_clear(self.misc.level, assets, coord);

            use CollisionType::*;
            self.dave.collision_point[i] = match col_type {
                Wall => false,
                Door => {
                    self.dave.check_door = true;
                    true
                }
                Pickup(x, y) => {
                    self.dave.check_pickup = Position {x, y};
                    true
                },
                _ => true,
            };
        }

        self.dave.on_ground = !self.dave.collision_point[4] || !self.dave.collision_point[5];
    }

    fn start_level(&mut self) {
        let start_pos = self.misc.level.start_position();

        self.dave.level_reset(start_pos);
        self.misc.view_x = 0;

        self.misc.has_trophy = false;
    }

    fn update_level(&mut self) {
        self.dave.jetpack_delay = self.dave.jetpack_delay.saturating_sub(1);

        if self.dave.check_door {
            if self.misc.has_trophy {
                if let Some(next) = self.misc.level.next() {
                    self.misc.level = next;
                    self.start_level();
                } else {
                    println!("You won with {} points!", self.misc.score);
                    self.misc.quit = true;
                }
            } else {
                self.dave.check_door = false;
            }
        }
    }

    pub fn update(&mut self, assets: &mut Assets) {
        self.check_dave_collision(assets);
        self.pickup_item(assets);
        self.dave.update_bullet(self.misc.level, self.misc.view_x, assets);
        self.dave.verify_input();
        self.dave.move_dave();
        self.scroll_screen();
        self.dave.apply_gravity(self.misc.level, assets);
        self.update_level();
    }

    pub fn input(&mut self, input: &Input) {
        self.dave.input(input);
    }
}

#[derive(Debug)]
pub struct Bullet {
    pub position: Position<i16>,
    pub direction: Direction,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CollisionType {
    None,
    Pickup(u8, u8),
    Wall,
    Door,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Position<T> {
    pub x: T,
    pub y: T,
}

impl<T: Add<T, Output=T>> Add<(T, T)> for Position<T> {
    type Output = Self;

    fn add(self, other: (T, T)) -> Self {
        Position {
            x: self.x + other.0,
            y: self.y + other.1,
        }
    }
}

pub fn is_clear(level: LevelId, assets: &Assets, pos: Position<i16>) -> CollisionType {
    let grid_x = pos.x as usize / TILE_SIZE as usize;
    let grid_y = pos.y as usize / TILE_SIZE as usize;

    let level = assets.get_level(level);
    let tile_type = level.tiles()[grid_y*100+grid_x];

    if tile_type.is_collidable() {
        CollisionType::Wall
    } else if tile_type.is_pickup() {
        CollisionType::Pickup(grid_x as u8, grid_y as u8)
    } else if tile_type.is_door() {
        CollisionType::Door
    } else {
        CollisionType::None
    }
}