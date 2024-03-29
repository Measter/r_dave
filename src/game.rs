use std::ops::Add;

use crate::{
    level::*, Result, TILE_SIZE,
    assets::Assets,
    input::Input,
    tileset::*,
    dave::*,
    monster::*,
};

pub struct Game {
    dave: Dave,
    misc: MiscParts,
    monsters: [Monster; 5],
    monster_bullet: Option<Bullet>,
}

#[derive(Debug)]
struct MiscParts {
    level: LevelId,
    view_x: i8,
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

    pub fn score(&self) -> u32 {
        self.misc.score
    }

    pub fn lives(&self) -> u8 {
        self.misc.lives
    }

    pub fn has_trophy(&self) -> bool {
        self.misc.has_trophy
    }

    pub fn has_gun(&self) -> bool {
        self.dave.has_gun
    }

    pub fn has_jetpack(&self) -> HasJetpack {
        self.dave.has_jetpack
    }

    pub fn view_x(&self) -> i8 {
        self.misc.view_x
    }

    pub fn dave(&self) -> &Dave {
        &self.dave
    }

    pub fn quit(&self) -> bool {
        self.misc.quit
    }

    pub fn monsters(&self) -> &[Monster] {
        &self.monsters
    }

    pub fn monster_bullet(&self) -> Option<&Bullet> {
        self.monster_bullet.as_ref()
    }
}

impl Game {
    pub fn init() -> Result<Self> {
        let mut game = Game {
            misc: MiscParts {
                level: LevelId::first_level(),
                view_x: 0,
                scroll_x: 0,
                score: 0,
                lives: 3,
                has_trophy: false,
                quit: false,
            },

            dave: Dave::init(),
            monsters: [Monster::init_dead(), Monster::init_dead(), Monster::init_dead(), Monster::init_dead(), Monster::init_dead()],
            monster_bullet: None,
        };

        game.start_level();

        Ok(game)
    }

    fn scroll_screen(&mut self) {
        match self.dave.position.x as i16 - self.misc.view_x as i16 {
            18 ..= 255 => self.misc.scroll_x = 15,
            -255 ..= 1    => self.misc.scroll_x = -15,
            _ => {}
        }

        if self.misc.scroll_x > 0 {
            self.misc.view_x = (self.misc.view_x + 1).min(80);
            self.misc.scroll_x -= 1;
        }

        if self.misc.scroll_x < 0 {
            self.misc.view_x = self.misc.view_x.checked_sub(1).map(|p| p.max(0)).unwrap_or(0);
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
            TileId::TILE_JETPACK => self.dave.has_jetpack = HasJetpack::Yes(255),
            TileId::TILE_GUN => self.dave.has_gun = true,
            t if t.is_trophy() => {
                self.add_score(1000);
                self.misc.has_trophy = true;
            },

            TileId::TILE_SCORE_BLUE_GEM => self.add_score(100),
            TileId::TILE_SCORE_ORB      => self.add_score(50),
            TileId::TILE_SCORE_RED_GEM  => self.add_score(150),
            TileId::TILE_SCORE_CROWN    => self.add_score(300),
            TileId::TILE_SCORE_RING     => self.add_score(200),
            TileId::TILE_SCORE_SCEPTER  => self.add_score(500),
            _ => {}
        }

        *tile_type = TileId::TILE_BLANK;
        self.dave.check_pickup = Default::default();
    }

    fn start_level(&mut self) {
        let start_pos = self.misc.level.start_position();

        self.dave.new_level(start_pos);

        self.monsters = self.misc.level.monsters();

        self.misc.view_x = 0;
        self.misc.has_trophy = false;
    }

    fn update_bullets(&mut self, assets: &Assets) {
        let dave_pos = self.dave.position;
        if let Some(bullet) = self.dave.bullet_mut() {
            match bullet.update_bullet(dave_pos, &self.monsters, &self.misc, assets) {
                (CollisionType::Wall, _) | (_, false) => *self.dave.bullet_mut() = None,
                (CollisionType::Monster(id), _) => {
                    *self.dave.bullet_mut() = None;
                    self.monsters[id].kill();
                }
                _ => {},
            }
        }

        if let Some(bullet) = &mut self.monster_bullet {
            match bullet.update_bullet(dave_pos, &self.monsters, &self.misc, assets) {
                (CollisionType::Wall, _) | (_, false) => self.monster_bullet = None,
                (CollisionType::Dave, _) if self.dave.is_alive() => {
                    self.monster_bullet = None;
                    self.dave.kill();
                }
                _ => {},
            }
        }
    }

    fn update_level(&mut self) {
        if self.dave.check_door {
            if self.misc.has_trophy {
                self.add_score(2000);
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

        if self.dave.is_dead() {
            if self.misc.lives != 0 {
                self.misc.lives = self.misc.lives.saturating_sub(1);
                self.dave.level_restart(self.misc.level.start_position());
            } else {
                self.misc.quit = true;
            }
        }

        if self.dave.is_alive() {
            for m in self.monsters.iter_mut().filter(|m| m.is_alive()) {
                if self.dave.position == m.position() {
                    self.dave.kill();
                    m.kill();
                    break;
                }
            }
        }
    }

    fn add_score(&mut self, new_score: u32) {
        if self.misc.score / 20_000 != (self.misc.score + new_score) / 20_000 {
            self.misc.lives += 1;
        }

        self.misc.score += new_score;
    }

    pub fn update(&mut self, assets: &mut Assets) {
        self.dave.check_collision(self.misc.level, assets);
        self.pickup_item(assets);
        self.dave.verify_input();
        self.dave.move_dave();

        for m in self.monsters.iter_mut() {
            m.move_monster(self.misc.level, assets);

            if self.monster_bullet.is_none() {
                self.monster_bullet = m.try_fire_bullet(self.dave.pixel_position, self.misc.view_x);
            }

            m.update();
        }

        self.update_bullets(assets);

        self.scroll_screen();
        self.dave.apply_gravity(self.misc.level, assets);
        self.dave.update();
        self.update_level();
    }

    pub fn input(&mut self, input: &Input) {
        self.dave.input(input);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Left,
    Middle,
    Right
}

#[derive(Debug, Copy, Clone)]
pub enum BulletSource {
    Dave,
    Monster,
}

#[derive(Debug)]
pub struct Bullet {
    pub source: BulletSource,
    pub position: Position<i16>,
    pub direction: Direction,
}

impl Bullet {
    fn update_bullet(&mut self, dave_pos: Position<i8>, monsters: &[Monster], level_misc: &MiscParts, assets: &Assets) -> (CollisionType, bool) {
        let dir_mult = match self.direction {
            Direction::Right | Direction::Middle => 1,
            Direction::Left => -1,
        };

        self.position.x += dir_mult * 4;

        let grid_x = (self.position.x / TILE_SIZE as i16) as i8;
        let grid_y = (self.position.y / TILE_SIZE as i16) as i8;
        let hit = is_clear(level_misc.level, assets, self.position);
        let visible = is_visible(grid_x, level_misc.view_x);

        match (hit, visible) {
            (CollisionType::Wall, _) | (_, false) => (hit, visible),
            _ => {
                match self.source {
                    BulletSource::Dave => {
                        let monsters = monsters.iter()
                            .enumerate()
                            .filter(|(_,m)| m.is_alive());

                        for (i, m) in monsters {
                            if (m.position().x == grid_x || m.position().x + 1 == grid_x)
                                && (m.position().y == grid_y || m.position().y + 1 == grid_y )
                            {
                                return (CollisionType::Monster(i), visible);
                            }
                        }
                    },
                    BulletSource::Monster => {
                        if grid_x == dave_pos.x && grid_y == dave_pos.y  {
                            return (CollisionType::Dave, visible);
                        }
                    }
                }

                (CollisionType::None, visible)
            },
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CollisionType {
    None,
    Pickup(u8, u8),
    Wall,
    Hazard,
    Door,
    Dave,
    Monster(usize),
    Climbable,
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

    if grid_x > 99 || grid_y > 9 {
        return CollisionType::None;
    }

    let level = assets.get_level(level);
    let tile_type = level.tiles()[grid_y*100+grid_x];

    if tile_type.is_collidable() {
        CollisionType::Wall
    } else if tile_type.is_hazard() {
        CollisionType::Hazard
    } else if tile_type.is_pickup() {
        CollisionType::Pickup(grid_x as u8, grid_y as u8)
    } else if tile_type.is_door() {
        CollisionType::Door
    } else if tile_type.is_climbable() {
        CollisionType::Climbable
    } else {
        CollisionType::None
    }
}

pub fn is_visible(pos_x: i8, view_x: i8) -> bool {
    if pos_x < view_x {
        false
    } else if pos_x - view_x < 20 {
        true
    } else {
        false
    }
}