use crate::core;

use core::{Json, JsonError};

pub fn parse_true(source: &str) -> Result<(Json, &str), JsonError> {
    if let Some(rest) = source.strip_prefix("true") {
        Ok((Json::JsonBoolean(true), rest))
    } else {
        Err(JsonError::SyntaxError(format!("Expected 'true' but found '{}'", &source[..source.len().min(4)])))
    }
}

pub fn parse_false(source: &str) -> Result<(Json, &str), JsonError> {
    if let Some(rest) = source.strip_prefix("false") {
        Ok((Json::JsonBoolean(false), rest))
    } else {
        Err(JsonError::SyntaxError(format!("Expected 'false' but found '{}'", &source[..source.len().min(5)])))
    }
}

pub fn parse_null(source: &str) -> Result<(Json, &str), JsonError> {
    if let Some(rest) = source.strip_prefix("null") {
        Ok((Json::JsonNull, rest))
    } else {
        Err(JsonError::SyntaxError(format!("Expected 'null' but found '{}'", &source[..source.len().min(4)])))
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_true_valid() {
        let input = "true rest";
        let (val, tail) = parse_true(input).unwrap();
        assert_eq!(val, Json::JsonBoolean(true));
        assert_eq!(tail, " rest");
    }

    #[test]
    fn test_parse_true_invalid() {
        let input = "truX";
        let err = parse_true(input).unwrap_err();
        assert!(matches!(err, JsonError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_false_valid() {
        let input = "false next";
        let (val, tail) = parse_false(input).unwrap();
        assert_eq!(val, Json::JsonBoolean(false));
        assert_eq!(tail, " next");
    }

    #[test]
    fn test_parse_false_invalid() {
        let input = "falsX";
        let err = parse_false(input).unwrap_err();
        assert!(matches!(err, JsonError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_null_valid() {
        let input = "null,";
        let (val, tail) = parse_null(input).unwrap();
        assert_eq!(val, Json::JsonNull);
        assert_eq!(tail, ",");
    }

    #[test]
    fn test_parse_null_invalid() {
        let input = "nulX";
        let err = parse_null(input).unwrap_err();
        assert!(matches!(err, JsonError::SyntaxError(_)));
    }
}

