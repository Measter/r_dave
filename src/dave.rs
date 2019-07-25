use crate::{
    TILE_SIZE,
    game::*,
    assets::*,
    level::*,
    input::*,
};

#[derive(Debug)]
pub enum DaveState {
    Live {
        move_type: MovementType,

        left: MoveState,
        right: MoveState,
        toggle_jetpack: MoveState,
        fire: MoveState,

        last_direction: Direction,
        on_ground: bool,
        jetpack_delay: u8,

        collision_point: [bool; 8],
    },
    Dying {
        dead_timer: u8,
    },
    Dead,
}

#[derive(Debug)]
pub struct Dave {
    pub position: Position<u8>,
    pub pixel_position: Position<i16>,
    pub animation_tick: usize,

    state: DaveState,

    pub has_jetpack: HasJetpack,
    pub has_gun: bool,
    bullet: Option<Bullet>,

    pub check_pickup: Position<u8>,
    pub check_door: bool,
}

impl Dave {
    pub fn bullet(&self) -> Option<&Bullet> {
        self.bullet.as_ref()
    }

    pub fn bullet_mut(&mut self) -> &mut Option<Bullet> {
        &mut self.bullet
    }

    pub fn direction(&self) -> Direction {
        match self.state {
            DaveState::Live {last_direction, ..} => last_direction,
            _ => Direction::Middle,
        }
    }

    pub fn is_jetpacking(&self) -> bool {
        match &self.state {
            DaveState::Live {move_type, ..} => move_type.is_jetpack(),
            _ => false
        }
    }

    pub fn is_on_ground(&self) -> bool {
        match self.state {
            DaveState::Live {on_ground, ..} => on_ground,
            _ => false,
        }
    }

    pub fn is_alive(&self) -> bool {
        match &self.state {
            DaveState::Live {..} => true,
            _ => false,
        }
    }

    pub fn is_dead(&self) -> bool {
        match &self.state {
            DaveState::Dead => true,
            _ => false,
        }
    }
}

impl Dave {
    pub fn init() -> Dave {
        Dave {
            position: Default::default(),
            pixel_position: Default::default(),
            animation_tick: 1,

            state: DaveState::Live {
                move_type: MovementType::Walking {
                    jump: MoveState::None,
                    jump_timer: 0,
                },

                right: MoveState::None,
                left: MoveState::None,
                toggle_jetpack: MoveState::None,
                fire: MoveState::None,

                on_ground: true,
                last_direction: Direction::Middle,
                jetpack_delay: 0,

                collision_point: [false; 8],
            },
            has_jetpack: HasJetpack::No,
            has_gun: false,
            bullet: None,

            check_pickup: Default::default(),
            check_door: true,
        }
    }

    pub fn apply_gravity(&mut self, level: LevelId, assets: &Assets) {
        if let DaveState::Live {move_type, on_ground, ..} = &mut self.state {
            match &move_type {
                MovementType::Walking {jump, ..} if *jump != MoveState::Do && !*on_ground => {
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
    }

    pub fn move_dave(&mut self) {
        use MoveState::*;

        if let DaveState::Live {
            move_type,
            toggle_jetpack,
            jetpack_delay,
            collision_point,
            last_direction,
            right,
            left,
            fire,
            ..} = &mut self.state
        {
            if *toggle_jetpack == Do && *jetpack_delay == 0 {

                let next_move_type = match &move_type {
                    MovementType::Walking { jump, .. } => {
                        MovementType::Jetpack {
                            up: *jump,
                            down: None,
                        }
                    },
                    MovementType::Jetpack { up, .. } => {
                        MovementType::Walking {
                            jump: *up,
                            jump_timer: 0,
                        }
                    },
                };

                *move_type = next_move_type;
                *toggle_jetpack = None;
            }

            match move_type {
                MovementType::Walking { jump, jump_timer } => {
                    if *jump == Do {
                        if *jump_timer == 0 {
                            *jump_timer = 25;
                            *last_direction = Direction::Middle;
                        }

                        if collision_point[0] && collision_point[1] {
                            match *jump_timer {
                                0..=4 => {},
                                5..=10 => self.pixel_position.y -= 1,
                                _ => self.pixel_position.y -= 2
                            }
                        }

                        *jump_timer -= 1;

                        if *jump_timer == 0 {
                            *jump = None;
                        }
                    }
                },
                MovementType::Jetpack { up, down, .. } => {
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

            if *right == Do {
                self.pixel_position.x += 2;
                self.animation_tick += 1;
                *right = None;
                *last_direction = Direction::Right;
            }

            if *left == Do {
                self.pixel_position.x -= 2;
                self.animation_tick += 1;
                *left = None;
                *last_direction = Direction::Left;
            }

            if *fire == Do {
                let x = match *last_direction {
                    Direction::Right | Direction::Middle => self.pixel_position.x + 18,
                    Direction::Left => self.pixel_position.x - 8,
                };

                self.bullet = Some(Bullet {
                    source: BulletSource::Dave,
                    direction: *last_direction,
                    position: Position {
                        x,
                        y: self.pixel_position.y + 8,
                    }
                });

                *fire = None;
            }

            self.position.x = (self.pixel_position.x / TILE_SIZE as i16) as u8;
            self.position.y = (self.pixel_position.y / TILE_SIZE as i16) as u8;
        }
    }

    pub fn verify_input(&mut self) {
        use MoveState::*;

        if let DaveState::Live{
            move_type,
            left,
            right,
            fire,
            toggle_jetpack,
            jetpack_delay,
            on_ground,
            collision_point,
            ..} = &mut self.state
        {
            if *left == Try && collision_point[6] && collision_point[7] {
                *left = Do;
            }

            if *right == Try && collision_point[2] && collision_point[3] {
                *right = Do;
            }

            match &mut *move_type {
                MovementType::Walking { jump, ..} => {
                    if *jump == Try && *on_ground && collision_point[0] && collision_point[1] {
                        *jump = Do;
                    }
                },
                MovementType::Jetpack { up, down, ..} => {
                    if *down == Try && collision_point[4] && collision_point[5] {
                        *down = Do;
                    }

                    if *up == Try && collision_point[0] && collision_point[1] {
                        *up = Do;
                    }
                },
            }

            if self.bullet.is_none() && *fire == Try && self.has_gun {
                *fire = Do;
            }

            if self.has_jetpack != HasJetpack::No && *toggle_jetpack == Try {
                if *jetpack_delay == 0 {
                    *toggle_jetpack = Do;
                    *jetpack_delay = 10;
                } else {
                    *toggle_jetpack = None;
                }
            }
        }
    }

    pub fn input(&mut self, input: &Input) {
        if let DaveState::Live {
            move_type,
            left,
            right,
            fire,
            toggle_jetpack,
            on_ground,
            ..
        } = &mut self.state
        {
            match move_type {
                MovementType::Walking {jump, ..} => {
                    if input.jump() && *jump == MoveState::None && *on_ground {
                        *jump = MoveState::Try;
                    }
                },
                MovementType::Jetpack {up, down, ..} => {
                    if input.jump() {
                        *up = MoveState::Try;
                    }

                    if input.down() {
                        *down = MoveState::Try;
                    }
                }
            }

            if input.right() {
                *right = MoveState::Try;
            }

            if input.left() {
                *left = MoveState::Try;
            }

            if input.fire() && self.bullet.is_none() {
                *fire = MoveState::Try;
            }

            if input.toggle_jetpack() {
                *toggle_jetpack = MoveState::Try;
            }
        }
    }

    pub fn level_restart(&mut self, start_pos: Position<u8>) {
        self.position = start_pos;
        self.pixel_position = Position {
            x: start_pos.x as i16 * TILE_SIZE as i16,
            y: start_pos.y as i16 * TILE_SIZE as i16,
        };

        self.state = DaveState::Live {
            right: MoveState::None,
            left: MoveState::None,
            toggle_jetpack: MoveState::None,
            fire: MoveState::None,

            on_ground: false,
            jetpack_delay: 0,
            collision_point: [false; 8],

            move_type: MovementType::Walking {
                jump: MoveState::None,
                jump_timer: 0,
            },
            last_direction: Direction::Middle,
        };
    }

    pub fn new_level(&mut self, start_pos: Position<u8>) {
        self.level_restart(start_pos);

        self.has_gun = false;
        self.has_jetpack = HasJetpack::No;
        self.check_door = false;
        self.bullet = None;
    }

    pub fn update(&mut self) {
        match &mut self.state {
            DaveState::Live {jetpack_delay, move_type, ..} => {
                *jetpack_delay = jetpack_delay.saturating_sub(1);
                if let (MovementType::Jetpack {up, ..}, HasJetpack::Yes(fuel)) = (&mut *move_type, &mut self.has_jetpack) {
                    *fuel = fuel.saturating_sub(1);
                    if *fuel == 0 {
                        self.has_jetpack = HasJetpack::No;
                        *move_type =  MovementType::Walking {
                            jump: *up,
                            jump_timer: 0,
                        };
                    }
                }
            },
            DaveState::Dying {dead_timer} => {
                *dead_timer = dead_timer.saturating_sub(1);
                if *dead_timer == 0 {
                    self.state = DaveState::Dead;
                }
            },
            DaveState::Dead => {}
        }
    }

    pub fn check_collision(&mut self, level: LevelId, assets: &Assets) {
        let mut kill_dave = false;
        match &mut self.state {
            DaveState::Live {collision_point, on_ground, ..} => {
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
                    let col_type = is_clear(level, assets, coord);

                    use CollisionType::*;
                    collision_point[i] = match col_type {
                        Wall => false,
                        Hazard => {
                            kill_dave = true;
                            false
                        }
                        Door => {
                            self.check_door = true;
                            true
                        }
                        Pickup(x, y) => {
                            self.check_pickup = Position {x, y};
                            true
                        },
                        _ => true,
                    };
                }

                *on_ground = !collision_point[4] || !collision_point[5];
            },
            _ => {}
        }

        if kill_dave {
            self.kill();
        }
    }

    pub fn kill(&mut self) {
        match &self.state {
            DaveState::Live {..} => self.state = DaveState::Dying {dead_timer: 30},
            _ => panic!("Tried to kill a dead or dying Dave."),
        }
    }
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
pub enum HasJetpack {
    No,
    Yes(u8),
}

impl MovementType {
    fn is_jetpack(&self) -> bool {
        if let MovementType::Jetpack {..} = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MoveState {
    None,
    Try,
    Do,
}