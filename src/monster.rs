use crate::{
    TILE_SIZE,
    tileset::*,
    game::*,
    assets::*,
    level::*,
};

#[derive(Debug)]
pub enum MonsterState {
    Live {
        position: Position<i8>,
        pixel_position: Position<i16>,
        tile_id: TileId,
        path_index: MonsterPathIndex,
        next_px: Position<i16>,
    },
    Dying {
        position: Position<i8>,
        pixel_position: Position<i16>,
        dead_timer: u8,
    },
    Dead
}

#[derive(Debug)]
pub struct Monster {
    state: MonsterState,
}

impl Monster {
    pub fn tile_id(&self) -> TileId {
        match self.state {
            MonsterState::Live {tile_id, ..} => tile_id,
            MonsterState::Dying {..} => TileId::TILE_MONSTER_DYING,
            MonsterState::Dead => TileId::TILE_BLANK,
        }
    }

    pub fn position(&self) -> Position<i8> {
        match self.state {
            MonsterState::Live {position, ..} => position,
            MonsterState::Dying {position, ..} => position,
            _ => Default::default(),
        }
    }

    pub fn pixel_position(&self) -> Position<i16> {
        match self.state {
            MonsterState::Live {pixel_position, ..} => pixel_position,
            MonsterState::Dying {pixel_position, ..} => pixel_position,
            _ => Default::default(),
        }
    }

    pub fn is_alive(&self) -> bool {
        match self.state {
            MonsterState::Live {..} => true,
            _ => false,
        }
    }

    pub fn is_not_dead(&self) -> bool {
        match self.state {
            MonsterState::Dead => false,
            _ => true,
        }
    }
}

impl Monster {
    pub fn init_live(tile_id: TileId, pos: Position<i8>) -> Monster {
        Monster {
            state: MonsterState::Live {
                position: pos,
                pixel_position: Position {
                    x: pos.x as i16 * TILE_SIZE as i16,
                    y: pos.y as i16 * TILE_SIZE as i16,
                },
                tile_id,
                path_index: MonsterPathIndex::START,
                next_px: Position { x: 0, y: 0 },
            }
        }
    }

    pub fn init_dead() -> Monster {
        Monster {
            state: MonsterState::Dead,
        }
    }

    pub fn move_monster(&mut self, level: LevelId, assets: &Assets) {
        match &mut self.state {
            MonsterState::Live {position, pixel_position, next_px, path_index, ..} => {
                let path = assets.get_level(level).path();

                for _ in 0..2 {
                    if next_px.x == 0 && next_px.y == 0 {
                        *next_px = path[*path_index];
                        *path_index = path_index.next();
                    }


                    if *next_px == MonsterPath::PATH_END {
                        let start = MonsterPathIndex::START;
                        *next_px = path[start];
                        *path_index = start.next();
                    }

                    if next_px.x < 0 {
                        pixel_position.x -= 1;
                        next_px.x += 1;
                    } else if next_px.x > 0 {
                        pixel_position.x += 1;
                        next_px.x -= 1;
                    }

                    if next_px.y < 0 {
                        pixel_position.y -= 1;
                        next_px.y += 1;
                    } else if next_px.y > 0 {
                        pixel_position.y += 1;
                        next_px.y -= 1;
                    }
                }

                position.x = (pixel_position.x / TILE_SIZE as i16) as i8;
                position.y = (pixel_position.y / TILE_SIZE as i16) as i8;
            },
            _ => {},
        }
    }

    pub fn try_fire_bullet(&mut self, dave_pos: Position<i16>, view_x: i8) -> Option<Bullet> {
        match &mut self.state {
            MonsterState::Live {position, pixel_position, ..} if is_visible(position.x, view_x) => {
                let dir = if dave_pos.x < pixel_position.x {
                    Direction::Left
                } else {
                    Direction::Right
                };

                let x = match dir {
                    Direction::Right | Direction::Middle => pixel_position.x + 18,
                    Direction::Left => pixel_position.x - 8,
                };

                Some(Bullet {
                    source: BulletSource::Monster,
                    direction: dir,
                    position: Position {
                        x,
                        y: pixel_position.y + 8,
                    }
                })
            },
            _ => None,
        }
    }

    pub fn kill(&mut self) {
        match self.state {
            MonsterState::Live {position, pixel_position, ..} => {
                self.state = MonsterState::Dying {
                    position,
                    pixel_position,
                    dead_timer: 30,
                }
            },
            _ => panic!("Tried to kill dead or dying monster."),
        }
    }

    pub fn update(&mut self) {
        match &mut self.state {
            MonsterState::Dying {dead_timer, ..} => {
                *dead_timer = dead_timer.saturating_sub(1);
                if *dead_timer == 0 {
                    self.state = MonsterState::Dead;
                }
            }
            _ => {},
        }
    }
}