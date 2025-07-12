use std::collections::HashMap;

use crate::utils;
use crate::object;
use crate::array;
use crate::string;
use crate::literals;
use crate::number;

use utils::ignore_ws;
use object::parse_object;
use array::parse_array;
use string::parse_string;
use literals::{parse_true, parse_false, parse_null};
use number::parse_number;


#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    JsonObject(HashMap<String, Json>),
    JsonArray(Vec<Json>),
    JsonString(String),
    JsonNumber(f64),
    JsonBoolean(bool),
    JsonNull,
    JsonEmpty
}

impl Json {
    pub fn new_object() -> Self {
        Json::JsonObject(HashMap::new())
    }
    pub fn new_array() -> Self {
        Json::JsonArray(Vec::new())
    }
    pub fn dump(&self) -> String {
        match self {
            Json::JsonObject(fields) => {
                let mut result = String::from("{");
                for (key, value) in fields {
                    result.push_str(&format!("\"{}\":{},", key, value.dump()));
                }
                result.pop(); // Remove last comma
                result.push('}');
                result
            }
            Json::JsonArray(elements) => {
                let mut result = String::from("[");
                for element in elements {
                    result.push_str(&format!("{},", element.dump()));
                }
                result.pop(); // Remove last comma
                result.push(']');
                result
            }
            Json::JsonString(s) => format!("\"{}\"", s),
            Json::JsonNumber(n) => n.to_string(),
            Json::JsonBoolean(b) => b.to_string(),
            Json::JsonNull => "null".to_string(),
            Json::JsonEmpty => "".to_string()
        }
    }
}

pub fn parse(source: &str) -> Result<(Json, &str), JsonError>{
    let s = ignore_ws(source);  
    match s.chars().next() {
        Some('{') => parse_object(&s),
        Some('[') => parse_array(&s),
        Some('"') => parse_string(&s),
        Some('t') => parse_true(&s),
        Some('f') => parse_false(&s),
        Some('n') => parse_null(&s),
        Some(_) => parse_number(&s),
        None => Ok((Json::JsonEmpty, ""))
    }  
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    LexicalError(String),
    SyntaxError(String)
}