use piston::{
    input::{keyboard::Key, UpdateArgs},
};

use crate::{
    level::*,
    Result,
};

pub struct Game {
    current_level: LevelId,
    view_x: u8,
    scroll_x: i8
}

impl Game {
    pub fn current_level(&self) -> LevelId {
        self.current_level
    }

    pub fn view_x(&self) -> u8 {
        self.view_x
    }
}

impl Game {
    pub fn init() -> Result<Self> {
        Ok(Game {
            current_level: LevelId::first_level(),
            view_x: 0,
            scroll_x: 0,
        })
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        if self.scroll_x > 0 {
            self.view_x = (self.view_x + 1).min(80);
            self.scroll_x -= 1;
        }

        if self.scroll_x < 0 {
            self.view_x = self.view_x.checked_sub(1).unwrap_or(0);
            self.scroll_x += 1;
        }
    }

    pub fn input(&mut self, key: Key) {
        match key {
            Key::Right => self.scroll_x = 15,
            Key::Left => self.scroll_x = -15,
            Key::Up => self.current_level = self.current_level.next(),
            Key::Down => self.current_level = self.current_level.prev(),
            _ => {}
        }
    }
}