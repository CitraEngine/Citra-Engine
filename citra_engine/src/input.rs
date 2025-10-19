/*
Citra Engine - A Nintendo 3DS first game engine
Copyright (C) 2025  Citra Engine

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

macro_rules! BIT {
    ($n:expr) => {
        1 << $n
    };
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputBits {
    KeyA = BIT!(0),
    KeyB = BIT!(1),
    KeySelect = BIT!(2),
    KeyStart = BIT!(3),
    KeyDRight = BIT!(4),
    KeyDLeft = BIT!(5),
    KeyDUp = BIT!(6),
    KeyDDown = BIT!(7),
    KeyR = BIT!(8),
    KeyL = BIT!(9),
    KeyX = BIT!(10),
    KeyY = BIT!(11),
    KeyTouch = BIT!(20),
    KeyCPadRight = BIT!(28),
    KeyCPadLeft = BIT!(29),
    KeyCPadUp = BIT!(30),
    KeyCPadDown = BIT!(31),

    // Generic catch-all directions
    KeyUp = Self::KeyDUp as u32 | Self::KeyCPadUp as u32,
    KeyDown = Self::KeyDDown as u32 | Self::KeyCPadDown as u32,
    KeyLeft = Self::KeyDLeft as u32 | Self::KeyCPadLeft as u32,
    KeyRight = Self::KeyDRight as u32 | Self::KeyCPadRight as u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputState {
    keys_held: u32,
    keys_down: u32,
    keys_up: u32,
}