use std::collections::VecDeque;
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    Mouse(MouseButton),
    Key(KeyboardKey),
}

impl From<MouseButton> for Input {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}

impl From<KeyboardKey> for Input {
    fn from(value: KeyboardKey) -> Self {
        Self::Key(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputChange {
    Released,
    Pressed,
}

pub struct InputEvents {
    events: Vec<(Input, Option<InputChange>)>,
}

impl InputEvents {
    pub const fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub fn check(&mut self, rl: &RaylibHandle, inputs: impl IntoIterator<Item = Input>) {
        for input in inputs {
            let event = match input {
                Input::Mouse(btn) =>  {
                    if rl.is_mouse_button_pressed(btn) {
                        Some(InputChange::Pressed)
                    } else if rl.is_mouse_button_released(btn) {
                        Some(InputChange::Released)
                    } else { None }
                }

                Input::Key(key) => {
                    if rl.is_key_pressed(key) {
                        Some(InputChange::Pressed)
                    } else if rl.is_key_released(key) {
                        Some(InputChange::Released)
                    } else { None }
                }
            };

            let index = if let Some(pos) = self.events.iter().position(|(k, _)| *k == input) {
                pos
            } else {
                self.events.push((input, None));
                self.events.len() - 1
            };

            self.events[index].1 = event;
        }
    }

    #[must_use = "returns the element without any side effects"]
    pub fn peek(&self, input: Input) -> Option<InputChange> {
        self.events.iter().find_map(|(k, s)| if *k == input { *s } else { None })
    }

    pub fn pop(&mut self, input: Input) -> Option<InputChange> {
        self.events.iter_mut().find_map(|(k, s)| if *k == input { s.take() } else { None })
    }
}
