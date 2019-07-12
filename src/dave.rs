use crate::{
    TILE_SIZE,
    game::*,
    assets::*,
    level::*,
    input::*,
};

#[derive(Debug)]
pub struct Dave {
    pub position: Position<u8>,
    pub pixel_position: Position<i16>,

    move_type: MovementType,

    left: MoveState,
    right: MoveState,

    last_direction: Direction,
    pub on_ground: bool,

    toggle_jetpack: MoveState,
    pub has_jetpack: bool,
    pub jetpack_delay: u8,

    fire: MoveState,
    pub has_gun: bool,
    bullet: Option<Bullet>,

    pub collision_point: [bool; 8],
    pub check_pickup: Position<u8>,
    pub check_door: bool,
}

impl Dave {
    pub fn pixel_position(&self) -> Position<i16> {
        self.pixel_position
    }

    pub fn bullet(&self) -> Option<&Bullet> {
        self.bullet.as_ref()
    }

    pub fn direction(&self) -> Direction {
        self.last_direction
    }

    pub fn is_jetpacking(&self) -> bool {
        if let MovementType::Jetpack {..} = self.move_type {
            true
        } else {
            false
        }
    }
}

impl Dave {
    pub fn init() -> Dave {
        Dave {
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
        }
    }

    pub fn apply_gravity(&mut self, level: LevelId, assets: &Assets) {
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

    pub fn move_dave(&mut self) {
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

    pub fn verify_input(&mut self) {
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

    pub fn update_bullet(&mut self, level: LevelId, view_x: u8, assets: &Assets) {
        if let Some(bullet) = &mut self.bullet {
            let dir_mult = match bullet.direction {
                Direction::Right | Direction::Middle => 1,
                Direction::Left => -1,
            };

            bullet.position.x += dir_mult * 4;

            let grid_x = bullet.position.x / TILE_SIZE as i16;
            let hit = is_clear(level, assets, bullet.position) == CollisionType::Wall;
            let gone = (grid_x - view_x as i16) < 1 || (grid_x - view_x as i16) > 20;

            if hit || gone {
                self.bullet = None;
            }
        }
    }

    pub fn input(&mut self, input: &Input) {
        match &mut self.move_type {
            MovementType::Walking {jump, ..} => {
                if input.jump() && *jump == MoveState::None && self.on_ground {
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
            self.right = MoveState::Try;
        }

        if input.left() {
            self.left = MoveState::Try;
        }

        if input.fire() && self.bullet.is_none() {
            self.fire = MoveState::Try;
        }

        if input.toggle_jetpack() {
            self.toggle_jetpack = MoveState::Try;
        }
    }

    pub fn level_reset(&mut self, start_pos: Position<u8>) {
        self.position = start_pos;
        self.pixel_position = Position {
            x: start_pos.x as i16 * TILE_SIZE as i16,
            y: start_pos.y as i16 * TILE_SIZE as i16,
        };

        self.right = MoveState::None;
        self.left = MoveState::None;
        self.move_type = MovementType::Walking {
            jump: MoveState::None,
            jump_timer: 0,
        };
        self.has_gun = false;
        self.has_jetpack = false;
        self.check_door = false;
        self.last_direction = Direction::Middle;
        self.bullet = None;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Left,
    Middle,
    Right
}

#[derive(Debug)]
pub enum MovementType {
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
pub enum MoveState {
    None,
    Try,
    Do,
}