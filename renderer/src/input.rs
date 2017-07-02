// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::iter;
use std::marker::PhantomData;

// External Dependencies ------------------------------------------------------
use glutin::{MouseButton, VirtualKeyCode};


// Traits ---------------------------------------------------------------------
pub trait AdvanceableState {
    fn advance(&self) -> Self where Self: Sized;
    fn reset(&self) -> Self where Self: Sized;
    fn was_pressed(&self) -> bool where Self: Sized;
    fn was_released(&self) -> bool where Self: Sized;
    fn is_pressed(&self) -> bool where Self: Sized;
    fn is_released(&self) -> bool where Self: Sized;
}

pub trait WithPosition {
}


// Keyboad --------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq)]
pub enum Key {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    I = 8,
    J = 9,
    K = 10,
    L = 11,
    M = 12,
    N = 13,
    O = 14,
    P = 15,
    Q = 16,
    R = 17,
    S = 18,
    T = 19,
    U = 20,
    V = 21,
    W = 22,
    X = 23,
    Y = 24,
    Z = 25,
    Space = 27,
    Backspace = 28,
    Tab = 29,
    Key1 = 30,
    Key2 = 31,
    Key3 = 32,
    Key4 = 33,
    Key5 = 34,
    Key6 = 35,
    Key7 = 36,
    Key8 = 37,
    Key9 = 38,
    Key0 = 39,
    Return = 40,
    Escape = 41,
    LShift = 42,
    Unknown = 43
}

impl From<VirtualKeyCode> for Key {
    fn from(code: VirtualKeyCode) -> Self {
        match code {
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::B => Key::B,
            VirtualKeyCode::C => Key::C,
            VirtualKeyCode::D => Key::D,
            VirtualKeyCode::E => Key::E,
            VirtualKeyCode::F => Key::F,
            VirtualKeyCode::G => Key::G,
            VirtualKeyCode::H => Key::H,
            VirtualKeyCode::I => Key::I,
            VirtualKeyCode::J => Key::J,
            VirtualKeyCode::K => Key::K,
            VirtualKeyCode::L => Key::L,
            VirtualKeyCode::M => Key::M,
            VirtualKeyCode::N => Key::N,
            VirtualKeyCode::O => Key::O,
            VirtualKeyCode::P => Key::P,
            VirtualKeyCode::Q => Key::Q,
            VirtualKeyCode::R => Key::R,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::T => Key::T,
            VirtualKeyCode::U => Key::U,
            VirtualKeyCode::V => Key::V,
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::X => Key::X,
            VirtualKeyCode::Y => Key::Y,
            VirtualKeyCode::Z => Key::Z,
            VirtualKeyCode::Space => Key::Space,
            VirtualKeyCode::Back => Key::Backspace,
            VirtualKeyCode::Tab => Key::Tab,
            VirtualKeyCode::Key1 => Key::Key1,
            VirtualKeyCode::Key2 => Key::Key2,
            VirtualKeyCode::Key3 => Key::Key3,
            VirtualKeyCode::Key4 => Key::Key4,
            VirtualKeyCode::Key5 => Key::Key5,
            VirtualKeyCode::Key6 => Key::Key6,
            VirtualKeyCode::Key7 => Key::Key7,
            VirtualKeyCode::Key8 => Key::Key8,
            VirtualKeyCode::Key9 => Key::Key9,
            VirtualKeyCode::Key0 => Key::Key0,
            VirtualKeyCode::Return => Key::Return,
            VirtualKeyCode::Escape => Key::Escape,
            VirtualKeyCode::LShift => Key::LShift,
            _ => Key::Unknown
        }
    }
}

impl Into<usize> for Key {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum KeyState {
    WasPressed,
    Pressed,
    WasReleased,
    Released
}

impl AdvanceableState for KeyState {

    fn advance(&self) -> Self {
        match *self {
            KeyState::WasReleased => KeyState::Released,
            KeyState::WasPressed => KeyState::Pressed,
            _ => *self
        }
    }

    fn reset(&self) -> Self {
        match *self {
            KeyState::Pressed => KeyState::WasReleased,
            _ => *self
        }
    }

    fn was_pressed(&self) -> bool where Self: Sized {
        *self == KeyState::WasPressed
    }

    fn was_released(&self) -> bool where Self: Sized {
        *self == KeyState::WasReleased
    }

    fn is_pressed(&self) -> bool where Self: Sized {
        *self == KeyState::Pressed || *self == KeyState::WasPressed
    }

    fn is_released(&self) -> bool where Self: Sized {
        *self == KeyState::Released || *self == KeyState::WasReleased
    }

}

impl Default for KeyState {
    fn default() -> Self {
        KeyState::Released
    }
}


// Mouse ----------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq)]
pub enum Button {
    Left = 0,
    Right = 1,
    Unknown = 2
}

impl From<MouseButton> for Button {
    fn from(code: MouseButton) -> Self {
        match code {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            _ => Button::Unknown
        }
    }
}

impl Into<usize> for Button {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ButtonState {
    WasPressed(i32, i32),
    Pressed(i32, i32),
    WasReleased(i32, i32),
    Released(i32, i32),
    Position(i32, i32)
}

impl ButtonState {

    pub fn position(&self) -> (i32, i32) {
        match *self {
            ButtonState::WasPressed(x, y)
            | ButtonState::Pressed(x, y)
            | ButtonState::WasReleased(x, y)
            | ButtonState::Released(x, y)
            | ButtonState::Position(x, y) => (x, y)
        }
    }

}

impl AdvanceableState for ButtonState {

    fn advance(&self) -> Self {
        match *self {
            ButtonState::WasReleased(x, y) => ButtonState::Released(x, y),
            ButtonState::WasPressed(x, y) => ButtonState::Pressed(x, y),
            _ => *self
        }
    }

    fn reset(&self) -> Self {
        match *self {
            ButtonState::Pressed(x, y) => ButtonState::WasReleased(x, y),
            _ => *self
        }
    }

    fn was_pressed(&self) -> bool where Self: Sized {
        if let ButtonState::WasPressed(_, _) = *self {
            true

        } else {
            false
        }
    }

    fn was_released(&self) -> bool where Self: Sized {
        if let ButtonState::WasReleased(_, _) = *self {
            true

        } else {
            false
        }
    }

    fn is_pressed(&self) -> bool where Self: Sized {
        if let ButtonState::Pressed(_, _) = *self {
            true

        } else {
            self.was_pressed()
        }
    }

    fn is_released(&self) -> bool where Self: Sized {
        if let ButtonState::Released(_, _) = *self {
            true

        } else {
            self.was_released()
        }
    }

}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Released(-1, -1)
    }
}


// Input ----------------------------------------------------------------------
pub struct InputState<I, T, C> {
    index: PhantomData<I>,
    fields: Vec<T>,
    custom: C
}

impl<I, T, C> InputState<I, T, C> where T: Default + Clone + AdvanceableState, I: Into<usize> {

    pub fn new(size: usize, custom: C) -> Self {
        Self {
            fields: iter::repeat(T::default()).take(size).collect(),
            index: PhantomData,
            custom: custom
        }
    }

    pub fn was_pressed(&self, index: I) -> bool {
        self.fields[index.into()].was_pressed()
    }

    pub fn is_pressed(&self, index: I) -> bool {
        self.fields[index.into()].is_pressed()
    }

    pub fn was_released(&self, index: I) -> bool {
        self.fields[index.into()].was_released()
    }

    pub fn is_released(&self, index: I) -> bool {
        self.fields[index.into()].is_released()
    }

    pub fn set(&mut self, index: I, to: T) {
        self.fields[index.into()] = to;
    }

    pub fn advance(&mut self) {
        for value in &mut self.fields {
            *value = value.advance();
        }
    }

    pub fn reset(&mut self) {
        for value in &mut self.fields {
            *value = value.reset();
        }
    }

    pub fn get(&self, index: I) -> &T {
        &self.fields[index.into()]
    }

}

pub type Keyboard = InputState<Key, KeyState, ()>;
pub type Mouse = InputState<Button, ButtonState, (i32, i32)>;

impl Mouse {

    pub fn set_position(&mut self, position: (i32, i32)) {
        self.custom = position;
    }

    pub fn position(&self) -> (i32, i32) {
        self.custom
    }

}

