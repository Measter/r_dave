use piston_window::{
    ButtonArgs, ButtonState, Button,
    keyboard::Key,
};

// As far as I can tell, using Piston's ButtonArgs directly only makes
// the input happen once every button press. The result of that is the
// event happens once when you press it, and then repeatedly as the
// keyboard starts repeating the press event.
//
// For the game, I want the input to be considered constant from the
// time the button was pressed to the time it was released. Which leads
// us to this input "buffer" which causes the input to be constant from
// first press to release.

#[derive(Debug, Default)]
pub struct Input {
    right: bool,
    left: bool,
    down: bool,
    toggle_jetpack: bool,
    fire: bool,
    jump: bool,
}

impl Input {
    pub fn update(&mut self, button: ButtonArgs) {
        if let ButtonArgs{state, button: Button::Keyboard(key), ..} = button {
            match key {
                Key::Right => self.right = state == ButtonState::Press,
                Key::Left => self.left = state == ButtonState::Press,
                Key::Up => self.jump = state == ButtonState::Press,
                Key::Down => self.down = state == ButtonState::Press,
                Key::LCtrl => self.fire = state == ButtonState::Press,
                Key::LAlt => self.toggle_jetpack = state == ButtonState::Press,
                _ => {}
            }
        }
    }

    pub fn clear_toggles(&mut self) {
        self.toggle_jetpack = false;
    }

    pub fn right(&self) -> bool {
        self.right
    }

    pub fn left(&self) -> bool {
        self.left
    }

    pub fn jump(&self) -> bool {
        self.jump
    }

    pub fn down(&self) -> bool {
        self.down
    }

    pub fn toggle_jetpack(&self) -> bool {
        self.toggle_jetpack
    }

    pub fn fire(&self) -> bool {
        self.fire
    }
}