use std::ops::{Index, IndexMut};

use crate::core;
use crate::core::parse;
use crate::utils;

use core::{Json, JsonError};
use utils::ignore_ws;

fn parse_field(source: &str) -> Result<(String, &str), JsonError> {
    let s = ignore_ws(source);
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        if !(c=='"') {
            return Err(JsonError::SyntaxError(format!("Expected a '\"' as start of JSON object field but found '{}'", c)));
        }
    } else {
        return Err(JsonError::SyntaxError("Expected a '\"' start of JSON object field but found end of string".to_string()));
    }
    let mut field = String::new();
    let mut idx = 0;
    while let Some(c) = chars.next() {
        idx += c.len_utf8();
        if c == '"' {
            break;
        }
        field.push(c);
    }
    if !s[idx..].starts_with('"') {
        return Err(JsonError::SyntaxError("Expected a '\"' but reached end of string".to_string()));
    }
    idx += '"'.len_utf8();
    let tail = &s[idx..];
    Ok((field, tail))
}

pub fn parse_object(source: &str) -> Result<(Json, &str), JsonError> {
    let mut s: &str = ignore_ws(source);
    if !s.starts_with('{') {
        return Err(JsonError::SyntaxError("Expected '{' at start of object".to_string()));
    }
    s = &s[1..]; // consume '{'
    let mut json: Json = Json::new_object();

    loop {
        s = ignore_ws(s);
        if s.starts_with('}') {
            s = &s[1..]; // consume '}'
            break;
        }
        // Parse field
        let (field, tail) = parse_field(s)?;
        s = ignore_ws(tail);
        if !s.starts_with(':') {
            return Err(JsonError::SyntaxError("Expected ':' after field name".to_string()));
        }
        s = ignore_ws(&s[1..]);
        // Parse value
        let (value, tail) = parse(s)?;
        json[field] = value;
        s = ignore_ws(tail);
        if s.starts_with(',') {
            s = &s[1..];
            continue;
        } else if s.starts_with('}') {
            s = &s[1..];
            break;
        } else if s.is_empty() {
            return Err(JsonError::SyntaxError("Unexpected end of input in object".to_string()));
        } else {
            return Err(JsonError::SyntaxError(format!("Unexpected character in object: '{}'", s.chars().next().unwrap())));
        }
    }
    Ok((json, s))
}

impl Index<&str> for Json {
    type Output = Json;
    fn index(&self, key: &str) -> &Self::Output {
        match self {
            Json::JsonObject(map) => map.get(key).unwrap_or(&Json::JsonEmpty),
            _ => &Json::JsonEmpty,
        }
    }
}

impl Index<String> for Json {
    type Output = Json;
    fn index(&self, key: String) -> &Self::Output {
        self.index(key.as_str())
    }
}

impl Index<&String> for Json {
    type Output = Json;
    fn index(&self, key: &String) -> &Self::Output {
        self.index(key.as_str())
    }
}


impl IndexMut<&str> for Json {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        match self {
            Json::JsonObject(map) => map.entry(key.to_string()).or_insert(Json::JsonEmpty),
            _ => panic!("Not a JsonObject"),
        }
    }
}

impl IndexMut<String> for Json {
    fn index_mut(&mut self, key: String) -> &mut Self::Output {
        self.index_mut(key.as_str())
    }
}

impl IndexMut<&String> for Json {
    fn index_mut(&mut self, key: &String) -> &mut Self::Output {
        self.index_mut(key.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_valid() {
        let valid = "\"field\"";
        let (field, tail) = parse_field(valid).unwrap();
        assert_eq!(field, "field");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_parse_field_valid_ws() {
        let valid = "  \"field\"   ";
        let (field, tail) = parse_field(valid).unwrap();
        assert_eq!(field, "field");
        assert_eq!(tail, "   ");
    }

    #[test]
    fn test_parse_field_valid_tail() {
        let valid = "  \"field\"  : rest ";
        let (field, tail) = parse_field(valid).unwrap();
        assert_eq!(field, "field");
        assert_eq!(tail, "  : rest ");
    }

    #[test]
    fn test_parse_field_valid_ws_inside() {
        let valid = "\" field  \"";
        let (field, tail) = parse_field(valid).unwrap();
        assert_eq!(field, " field  ");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_parse_field_syntax_error_1st_quote() {
        let valid = " field\"  ";
        let result = parse_field(valid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_field_syntax_error_last_quote() {
        let valid = "  \"field ";
        let result = parse_field(valid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_object_valid() {
        let valid = r#"{"type": "type1", "type2": 0, "type3" : null}"#;
        let (obj, tail) = parse_object(valid).unwrap();
        assert_eq!(tail, "");
        assert_eq!(obj["type"], Json::JsonString("type1".to_string()));
        assert_eq!(obj["type2"], Json::JsonNumber(0.0));
        assert_eq!(obj["type3"], Json::JsonNull);
    }

    #[test]
    fn test_parse_object_empty() {
        let valid = r#"{}"#;
        let (obj, tail) = parse_object(valid).unwrap();
        assert_eq!(tail, "");
        // Should be an empty object
        if let Json::JsonObject(map) = obj {
            assert!(map.is_empty());
        } else {
            panic!("Expected JsonObject");
        }
    }

    #[test]
    fn test_parse_object_trailing_ws() {
        let valid = r#"{   "a": 1   }   "#;
        let (obj, tail) = parse_object(valid).unwrap();
        assert_eq!(obj["a"], Json::JsonNumber(1.0));
        assert_eq!(tail.trim(), "");
    }

    #[test]
    fn test_parse_object_missing_colon() {
        let invalid = r#"{"a" 1}"#;
        let result = parse_object(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_object_missing_value() {
        let invalid = r#"{"a": }"#;
        let result = parse_object(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_object_missing_field_quote() {
        let invalid = r#"{a: 1}"#;
        let result = parse_object(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_object_nested() {
        let valid = r#"{"outer": {"inner": 42}}"#;
        let (obj, tail) = parse_object(valid).unwrap();
        assert_eq!(tail, "");
        let inner = &obj["outer"];
        assert_eq!(inner["inner"], Json::JsonNumber(42.0));
    }
}