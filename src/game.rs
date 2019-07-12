use std::ops::Add;

use crate::{
    level::*, Result, TILE_SIZE,
    assets::Assets,
    input::Input,
    tileset::*,
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

            dave: Dave {
                position: Default::default(),
                pixel_position: Default::default(),

                move_type: MovementType::Walking {
                    jump: MoveState::None,
                    jump_timer: 0,
                },

                right: MoveState::None,
                left: MoveState::None,
                on_ground: true,
                last_direction: Direction::Middle,

                toggle_jetpack: MoveState::None,
                has_jetpack: false,
                jetpack_delay: 0,

                fire: MoveState::None,
                has_gun: false,
                bullet: None,

                collision_point: [false; 8],
                check_pickup: Default::default(),
                check_door: true,
            },
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

        self.dave.position = start_pos;
        self.dave.pixel_position = Position {
            x: start_pos.x as i16 * TILE_SIZE as i16,
            y: start_pos.y as i16 * TILE_SIZE as i16,
        };

        self.dave.right = MoveState::None;
        self.dave.left = MoveState::None;
        self.dave.move_type = MovementType::Walking {
            jump: MoveState::None,
            jump_timer: 0,
        };
        self.dave.has_gun = false;
        self.dave.has_jetpack = false;
        self.misc.has_trophy = false;
        self.dave.check_door = false;
        self.misc.view_x = 0;
        self.dave.last_direction = Direction::Middle;
        self.dave.bullet = None;
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

    fn update_dave_bullet(&mut self, assets: &Assets) {
        if let Some(bullet) = &mut self.dave.bullet {
            let dir_mult = match bullet.direction {
                Direction::Right | Direction::Middle => 1,
                Direction::Left => -1,
            };

            bullet.position.x += dir_mult * 4;

            let grid_x = bullet.position.x / TILE_SIZE as i16;
            let hit = is_clear(self.misc.level, assets, bullet.position) == CollisionType::Wall;
            let gone = (grid_x - self.misc.view_x as i16) < 1 || (grid_x - self.misc.view_x as i16) > 20;

            if hit || gone {
                self.dave.bullet = None;
            }
        }
    }

    pub fn update(&mut self, assets: &mut Assets) {
        self.check_dave_collision(assets);
        self.pickup_item(assets);
        self.update_dave_bullet(assets);
        self.dave.verify_input();
        self.dave.move_dave();
        self.scroll_screen();
        self.dave.apply_gravity(self.misc.level, assets);
        self.update_level();
    }

    pub fn input(&mut self, input: &Input) {
        match &mut self.dave.move_type {
            MovementType::Walking {jump, ..} => {
                if input.jump() && *jump == MoveState::None && self.dave.on_ground {
                    *jump = MoveState::Try;
                }
            },
            MovementType::Jetpack {up, down} => {
                if input.jump() {
                    *up = MoveState::Try;
                }

                if input.down() {
                    *down = MoveState::Try;
                }
            }
        }

        if input.right() {
            self.dave.right = MoveState::Try;
        }

        if input.left() {
            self.dave.left = MoveState::Try;
        }

        if input.fire() && self.dave.bullet.is_none() {
            self.dave.fire = MoveState::Try;
        }

        if input.toggle_jetpack() {
            self.dave.toggle_jetpack = MoveState::Try;
        }
    }
}

#[derive(Debug)]
pub struct Dave {
    pub position: Position<u8>,
    pub pixel_position: Position<i16>,

    move_type: MovementType,

    left: MoveState,
    right: MoveState,

    last_direction: Direction,
    on_ground: bool,

    toggle_jetpack: MoveState,
    has_jetpack: bool,
    jetpack_delay: u8,

    fire: MoveState,
    has_gun: bool,
    bullet: Option<Bullet>,

    collision_point: [bool; 8],
    check_pickup: Position<u8>,
    check_door: bool,
}

impl Dave {
    pub fn pixel_position(&self) -> Position<i16> {
        self.pixel_position
    }

    pub fn bullet(&self) -> Option<&Bullet> {
        self.bullet.as_ref()
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn direction(&self) -> Direction {
        self.last_direction
    }

    pub fn is_jetpack(&self) -> bool {
        if let MovementType::Jetpack {..} = self.move_type {
            true
        } else {
            false
        }
    }
}

impl Dave {
    fn apply_gravity(&mut self, level: LevelId, assets: &Assets) {
        match &self.move_type {
            MovementType::Walking {jump, ..} if *jump != MoveState::Do && !self.on_ground => {
                let is_clear = (
                    is_clear(level, assets, self.pixel_position + (4, 17)),
                    is_clear(level, assets, self.pixel_position + (10,17))
                );

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
            },
            _ => {}
        }
    }

    fn move_dave(&mut self) {
        use MoveState::*;

        if self.toggle_jetpack == Do && self.jetpack_delay == 0 {
            let next_move_type = match &self.move_type {
                MovementType::Walking { jump, .. } => {
                    MovementType::Jetpack {
                        up: *jump,
                        down: None
                    }
                },
                MovementType::Jetpack { up, ..} => {
                    MovementType::Walking {
                        jump: *up,
                        jump_timer: 0,
                    }
                },
            };

            self.move_type = next_move_type;
            self.toggle_jetpack = None;
        }

        match &mut self.move_type {
            MovementType::Walking { jump, jump_timer} => {
                if *jump == Do {
                    if *jump_timer == 0 {
                        *jump_timer = 25;
                        self.last_direction = Direction::Middle;
                    }

                    if self.collision_point[0] && self.collision_point[1] {
                        match *jump_timer {
                            0..=4 => {},
                            5..=10 => self.pixel_position.y -= 1,
                            _ => self.pixel_position.y -= 2
                        }

                        *jump_timer -= 1;
                    } else {
                        *jump_timer = 0;
                    }

                    if *jump_timer == 0 {
                        *jump = None;
                    }
                }
            },
            MovementType::Jetpack { up, down} => {
                if *up == Do {
                    self.pixel_position.y -= 2;
                    *up = None;
                }

                if *down == Do {
                    self.pixel_position.y += 2;
                    *down = None;
                }
            }
        }

        if self.right == Do {
            self.pixel_position.x += 2;
            self.right = None;
            self.last_direction = Direction::Right;
        }

        if self.left == Do {
            self.pixel_position.x -= 2;
            self.left = None;
            self.last_direction = Direction::Left;
        }

        if self.fire == Do {
            let x = match self.last_direction {
                Direction::Right | Direction::Middle => self.pixel_position.x + 18,
                Direction::Left => self.pixel_position.x - 8,
            };

            self.bullet = Some(Bullet {
                direction: self.last_direction,
                position: Position {
                    x,
                    y: self.pixel_position.y + 8,
                }
            });

            self.fire = None;
        }

        self.position.x = (self.pixel_position.x / TILE_SIZE as i16) as u8;
        self.position.y = (self.pixel_position.y / TILE_SIZE as i16) as u8;
    }

    fn verify_input(&mut self) {
        use MoveState::*;

        if self.left == Try && self.collision_point[6] && self.collision_point[7] {
            self.left = Do;
        }

        if self.right == Try && self.collision_point[2] && self.collision_point[3] {
            self.right = Do;
        }

        match &mut self.move_type {
            MovementType::Walking { jump, ..} => {
                if *jump == Try && self.on_ground && self.collision_point[0] && self.collision_point[1] {
                    *jump = Do;
                }
            },
            MovementType::Jetpack { up, down} => {
                if *down == Try && self.collision_point[4] && self.collision_point[5] {
                    *down = Do;
                }

                if *up == Try && self.collision_point[0] && self.collision_point[1] {
                    *up = Do;
                }
            },
        }

        if self.bullet.is_none() && self.fire == Try && self.has_gun {
            self.fire = Do;
        }

        if self.has_jetpack && self.toggle_jetpack == Try {
            if self.jetpack_delay == 0 {
                self.toggle_jetpack = Do;
                self.jetpack_delay = 10;
            } else {
                self.toggle_jetpack = None;
            }
        }
    }
}

#[derive(Debug)]
pub struct Bullet {
    pub position: Position<i16>,
    pub direction: Direction,
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Left,
    Middle,
    Right
}

#[derive(Debug)]
enum MovementType {
    Walking {
        jump: MoveState,
        jump_timer: u8,
    },
    Jetpack {
        up: MoveState,
        down: MoveState,
    },
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

fn is_clear(level: LevelId, assets: &Assets, pos: Position<i16>) -> CollisionType {
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