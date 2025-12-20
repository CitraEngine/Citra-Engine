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

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::util::visitors::StringVisitor;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodePath {
    pub path: Vec<String>,
}
impl From<String> for NodePath {
    fn from(value: String) -> Self {
        let mut out = Self {
            path: value.split('/').map(|s| s.to_string()).collect(),
        };
        if out.path[0].is_empty() {
            out.path.remove(0);
        }
        out
    }
}
impl fmt::Display for NodePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = "".to_string();
        for s in self.path.clone() {
            out += &format!("/{}", s)[..]
        }
        write!(f, "{}", out)
    }
}
impl NodePath {
    pub fn new() -> Self {
        NodePath { path: vec![] }
    }
    pub fn push<T>(&mut self, next: T)
    where
        T: ToString,
    {
        self.path.push(next.to_string());
    }
    pub fn extend(&mut self, other: &NodePath) {
        self.path.extend_from_slice(&other.path);
    }
    pub fn remove_first(&self) -> Self {
        let mut out = self.clone();
        out.path.remove(0);
        out
    }
}
impl Serialize for NodePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string()[..])
    }
}
impl<'de> Deserialize<'de> for NodePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from(
            deserializer.deserialize_string(StringVisitor {})?,
        ))
    }
}

#[test]
fn node_path_conversion() {
    let string = "/path/to/thing".to_string();
    let vector = vec!["path".to_string(), "to".to_string(), "thing".to_string()];
    let strtest = NodePath::from(string.clone());
    let vectest = NodePath {
        path: vector.clone(),
    };
    assert_eq!(strtest.path, vector.clone());
    assert_eq!(vectest.to_string(), string.clone());
}
