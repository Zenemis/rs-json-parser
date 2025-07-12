use std::ops::{Index, IndexMut};

use crate::core;
use crate::utils;

use core::{Json, JsonError};
use utils::ignore_ws;

pub fn parse_array(source: &str) -> Result<(Json, &str), JsonError> {
    todo!()
}


impl Index<usize> for Json {
    type Output = Json;
    fn index(&self, idx: usize) -> &Self::Output {
        match self {
            Json::JsonArray(vec) => vec.get(idx).unwrap_or(&Json::JsonEmpty),
            _ => &Json::JsonEmpty,
        }
    }
}


impl IndexMut<usize> for Json {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        match self {
            Json::JsonArray(vec) => vec.get_mut(idx).unwrap_or_else(|| panic!("Index out of bounds")),
            _ => panic!("Not a JsonArray"),
        }
    }
}