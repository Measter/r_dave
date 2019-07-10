use std::ops::Add;

use crate::{
    level::*, Result, TILE_SIZE,
    assets::Assets,
    input::Input,
    tileset::*,
};

pub struct Game {
    current_level: LevelId,
    view_x: u8,
    scroll_x: i8,
    dave: Dave,
}

impl Game {
    pub fn current_level(&self) -> LevelId {
        self.current_level
    }

    pub fn view_x(&self) -> u8 {
        self.view_x
    }

    pub fn dave(&self) -> &Dave {
        &self.dave
    }
}

impl Game {
    pub fn init() -> Result<Self> {
        let dave_pos = Position{x: 2, y: 8};

        Ok(Game {
            current_level: LevelId::first_level(),
            view_x: 0,
            scroll_x: 0,
            dave: Dave {
                position: dave_pos,
                pixel_position: Position {
                    x: dave_pos.x as i16 * TILE_SIZE as i16,
                    y: dave_pos.y as i16 * TILE_SIZE as i16
                },

                on_ground: true,

                right: MoveState::None,
                left: MoveState::None,
                jump: MoveState::None,
                jump_timer: 0,

                collision_point: [false; 8],
                check_pickup: Position { x: 0, y: 0 },
            },
        })
    }

    fn scroll_screen(&mut self) {
        if self.scroll_x > 0 {
            self.view_x = (self.view_x + 1).min(80);
            self.scroll_x -= 1;
        }

        if self.scroll_x < 0 {
            self.view_x = self.view_x.checked_sub(1).unwrap_or(0);
            self.scroll_x += 1;
        }
    }

    fn pickup_item(&mut self, assets: &mut Assets) {
        if self.dave.check_pickup.x == 0 && self.dave.check_pickup.y == 0 {
            return;
        }

        let level = assets.get_level_mut(self.current_level);
        let tile_type = &mut level.tiles_mut()[self.dave.check_pickup.y as usize * 100 + self.dave.check_pickup.x as usize];

        match tile_type {
            // Add score and special item cases here later.
            _ => {}
        }

        *tile_type = TileId::tile_blank();
        self.dave.check_pickup = Position { x: 0, y: 0 };
    }

    pub fn update(&mut self, assets: &mut Assets) {
        self.dave.check_collision(self.current_level, assets);
        self.pickup_item(assets);
        self.dave.verify_input();
        self.dave.move_dave();
        self.scroll_screen();
        self.dave.apply_gravity(self.current_level, assets);
    }

    pub fn input(&mut self, input: &Input) {
        if input.right() {
            self.dave.right = MoveState::Try;
        }

        if input.left() {
            self.dave.left = MoveState::Try;
        }

        if self.dave.jump == MoveState::None && input.jump() {
            self.dave.jump = MoveState::Try;
        }
    }
}

#[derive(Debug)]
pub struct Dave {
    pub position: Position<u8>,
    pub pixel_position: Position<i16>,

    right: MoveState,
    left: MoveState,
    jump: MoveState,
    jump_timer: u8,

    on_ground: bool,

    collision_point: [bool; 8],
    check_pickup: Position<u8>,
}

impl Dave {
    fn apply_gravity(&mut self, level: LevelId, assets: &Assets) {
        if self.jump != MoveState::Do && !self.on_ground {
            let is_clear = (self.is_clear(level, assets, self.pixel_position + (4, 17)),
                self.is_clear(level, assets, self.pixel_position + (10,17)));

            if is_clear != (CollisionType::Wall, CollisionType::Wall) {
                self.pixel_position.y += 2;
            } else {
                // Ensure that dave is aligned to the floor.
                let not_align = self.pixel_position.y % TILE_SIZE as i16;

                if not_align != 0 {
                    self.pixel_position.y = if not_align < 8 {
                        self.pixel_position.y - not_align
                    } else {
                        self.pixel_position.y + TILE_SIZE as i16 - not_align
                    };
                }
            }
        }
    }

    fn move_dave(&mut self) {
        use MoveState::*;

        if self.right == Do {
            self.pixel_position.x += 2;
            self.right = None;
        }

        if self.left == Do {
            self.pixel_position.x -= 2;
            self.left = None;
        }

        if self.jump == Do {
            if self.jump_timer == 0 {
                self.jump_timer = 20;
            }

            if self.collision_point[0] && self.collision_point[1] {
                if self.jump_timer > 5 {
                    self.pixel_position.y -= 2;
                } else {
                    self.pixel_position.y -= 1;
                }

                self.jump_timer -= 1;
            } else {
                self.jump_timer = 0;
            }

            if self.jump_timer == 0 {
                self.jump = None;
            }
        }
    }

    fn is_clear(&self, level: LevelId, assets: &Assets, pos: Position<i16>) -> CollisionType {
        let grid_x = pos.x as usize / TILE_SIZE as usize;
        let grid_y = pos.y as usize / TILE_SIZE as usize;

        let level = assets.get_level(level);
        let tile_type = level.tiles()[grid_y*100+grid_x];

        if tile_type.is_collidable() {
            CollisionType::Wall
        } else if tile_type.is_pickup() {
            CollisionType::Pickup(grid_x as u8, grid_y as u8)
        } else {
            CollisionType::None
        }
    }

    fn check_collision(&mut self, level: LevelId, assets: &Assets) {
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
            let coord = self.pixel_position + offset;
            let col_type = self.is_clear(level, assets, coord);

            self.collision_point[i] = match col_type {
                CollisionType::Wall => false,
                CollisionType::Pickup(x, y) => {
                    self.check_pickup = Position {x, y};
                    true
                },
                CollisionType::None => true,
            };
        }

        self.on_ground = !self.collision_point[4] || !self.collision_point[5];
    }

    fn verify_input(&mut self) {
        use MoveState::*;

        if self.left == Try && self.collision_point[6] && self.collision_point[7] {
            self.left = Do;
        }

        if self.right == Try && self.collision_point[2] && self.collision_point[3] {
            self.right = Do;
        }

        if self.jump == Try && self.on_ground && self.collision_point[0] && self.collision_point[1] {
            self.jump = Do;
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MoveState {
    None,
    Try,
    Do,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CollisionType {
    None,
    Pickup(u8, u8),
    Wall,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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