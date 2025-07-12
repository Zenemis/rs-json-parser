use std::ops::{Index, IndexMut};

use crate::core;
use crate::core::parse;
use crate::utils;

use core::{Json, JsonError};
use utils::ignore_ws;

pub fn parse_array(source: &str) -> Result<(Json, &str), JsonError> {
    let mut s: &str = ignore_ws(source);
    if !s.starts_with('[') {
        return Err(JsonError::SyntaxError("Expected '[' at start of array".to_string()));
    }
    s = &s[1..]; // consume '['
    let mut json: Vec<Json> = Vec::new();

    s = ignore_ws(s);
    if s.starts_with(']') {
        // Empty array
        s = &s[1..];
        return Ok((Json::JsonArray(json), s));
    }

    loop {
        let (value, tail) = parse(s)?;
        json.push(value);
        s = ignore_ws(tail);

        if s.starts_with(',') {
            s = &s[1..];
            s = ignore_ws(s);
            if s.starts_with(']') {
                // Trailing comma
                return Err(JsonError::SyntaxError("JSON Array should not end in a comma".to_string()));
            }
        } else if s.starts_with(']') {
            s = &s[1..];
            break;
        } else {
            return Err(JsonError::SyntaxError("JSON Array should separate values with commas".to_string()));
        }
    }
    Ok((Json::JsonArray(json), s))
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


mod tests {
    use std::string;

    use super::*;
    #[test]
    fn test_parse_array_valid() {
        let valid = r#"[null, true, "hello", 1]"#;
        let (arr, tail) = parse_array(valid).unwrap();
        assert_eq!(tail, "");
        assert_eq!(arr[0], Json::JsonNull);
        assert_eq!(arr[1], Json::JsonBoolean(true));
        assert_eq!(arr[2], Json::JsonString("hello".to_string()));
        assert_eq!(arr[3], Json::JsonNumber(1.0));
    }

    #[test]
    fn test_parse_array_empty() {
        let valid = r#"[]"#;
        let (arr, tail) = parse_array(valid).unwrap();
        assert_eq!(tail, "");
        // Should be an empty object
        if let Json::JsonArray(vec) = arr {
            assert!(vec.is_empty());
        } else {
            panic!("Expected JsonArray");
        }
    }

    #[test]
    fn test_parse_array_trailing_ws() {
        let valid = r#"[null]     "#;
        let (obj, tail) = parse_array(valid).unwrap();
        assert_eq!(obj[0], Json::JsonNull);
        assert_eq!(tail.trim(), "");
    }

    #[test]
    fn test_parse_array_missing_comma() {
        let invalid = r#"[null true]"#;
        let result = parse_array(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array_comma_at_end() {
        let invalid = r#"[null, true,]"#;
        let result = parse_array(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array_nested() {
        let valid = r#"[null, [true]]"#;
        let (arr, tail) = parse_array(valid).unwrap();
        assert_eq!(tail, "");
        assert_eq!(arr[0], Json::JsonNull);
        let inner = &arr[1];
        assert_eq!(inner[0], Json::JsonBoolean(true));
    }

    #[test]
    fn test_parse_array_with_whitespace_and_newlines() {
        let valid = "[  null ,\n true ,\t\"hi\"  , 42 ]";
        let (arr, tail) = parse_array(valid).unwrap();
        assert_eq!(tail, "");
        assert_eq!(arr[0], Json::JsonNull);
        assert_eq!(arr[1], Json::JsonBoolean(true));
        assert_eq!(arr[2], Json::JsonString("hi".to_string()));
        assert_eq!(arr[3], Json::JsonNumber(42.0));
    }

    #[test]
    fn test_parse_array_deeply_nested() {
        let valid = "[[[]], [null], [[true]]]";
        let (arr, tail) = parse_array(valid).unwrap();
        assert_eq!(tail, "");
        assert_eq!(arr[0][0], Json::JsonArray(vec![]));
        assert_eq!(arr[1][0], Json::JsonNull);
        assert_eq!(arr[2][0][0], Json::JsonBoolean(true));
    }

    #[test]
    fn test_parse_array_only_commas() {
        let invalid = "[,]";
        let result = parse_array(invalid);
        assert!(result.is_err());
    }
}