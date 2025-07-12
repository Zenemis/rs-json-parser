use std::str::CharIndices;
use std::str::FromStr;

use crate::core;

use core::{Json, JsonError};

pub fn parse_number(source: &str) -> Result<(Json, &str), JsonError> {
    let mut chars = source.char_indices().peekable();
    let mut end = 0;
    let mut has_digits = false;

    // Special early error return for leading dot
    if let Some((_, '.')) = chars.peek() {
        return Err(JsonError::SyntaxError("Leading dot is not allowed".to_string()));
    }
    

    // Optional minus
    if let Some((_, '-')) = chars.peek() {
        end += 1;
        chars.next();
    }

    // Integer part
    if let Some((_, '0')) = chars.peek() {
        end += 1;
        chars.next();
        has_digits = true;
        // Leading zero must not be followed by digit
        if let Some((_, c)) = chars.peek() {
            if c.is_ascii_digit() {
                return Err(JsonError::SyntaxError("Leading zeros are not allowed".to_string()));
            }
        }
    } else if let Some((_, c)) = chars.peek() {
        if c.is_ascii_digit() {
            has_digits = true;
            while let Some((i, c)) = chars.peek() {
                if c.is_ascii_digit() {
                    end = *i + c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }

    // Fractional part
    if let Some((_, '.')) = chars.peek() {
        end += 1;
        chars.next();
        let mut frac_digits = false;
        while let Some((i, c)) = chars.peek() {
            if c.is_ascii_digit() {
                end = *i + c.len_utf8();
                chars.next();
                frac_digits = true;
            } else {
                break;
            }
        }
        if !frac_digits {
            return Err(JsonError::SyntaxError("Expected digits after decimal point".to_string()));
        }
        has_digits = true;
    }

    // Exponent part
    if let Some((_, c)) = chars.peek() {
        if *c == 'e' || *c == 'E' {
            end += 1;
            chars.next();
            // Optional sign
            if let Some((_, c2)) = chars.peek() {
                if *c2 == '+' || *c2 == '-' {
                    end += 1;
                    chars.next();
                }
            }
            let mut exp_digits = false;
            while let Some((i, c)) = chars.peek() {
                if c.is_ascii_digit() {
                    end = *i + c.len_utf8();
                    chars.next();
                    exp_digits = true;
                } else {
                    break;
                }
            }
            if !exp_digits {
                return Err(JsonError::SyntaxError("Expected digits in exponent".to_string()));
            }
            has_digits = true;
        }
    }

    if !has_digits {
        return Err(JsonError::SyntaxError("No digits found in number".to_string()));
    }

    let (number_str, rest) = source.split_at(end);
    let num = number_str.parse::<f64>()
        .map_err(|err| JsonError::SyntaxError(format!("{}", err)))?;
    Ok((Json::JsonNumber(num), rest))
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_number_integer() {
        let input = "42 rest";
        let (val, tail) = parse_number(input).unwrap();
        assert_eq!(val, Json::JsonNumber(42.0));
        assert_eq!(tail, " rest");
    }

    #[test]
    fn test_parse_number_negative_integer() {
        let input = "-123,";
        let (val, tail) = parse_number(input).unwrap();
        assert_eq!(val, Json::JsonNumber(-123.0));
        assert_eq!(tail, ",");
    }

    #[test]
    fn test_parse_number_zero() {
        let input = "0]";
        let (val, tail) = parse_number(input).unwrap();
        assert_eq!(val, Json::JsonNumber(0.0));
        assert_eq!(tail, "]");
    }

    #[test]
    fn test_parse_number_leading_zero_float() {
        let input = "0.123}";
        let (val, tail) = parse_number(input).unwrap();
        assert_eq!(val, Json::JsonNumber(0.123));
        assert_eq!(tail, "}");
    }

    #[test]
    fn test_parse_number_float() {
        let input = "-12.34 ";
        let (val, tail) = parse_number(input).unwrap();
        assert_eq!(val, Json::JsonNumber(-12.34));
        assert_eq!(tail, " ");
    }

    #[test]
    fn test_parse_number_exponent() {
        let input = "6.022e23,";
        let (val, tail) = parse_number(input).unwrap();
        assert_eq!(val, Json::JsonNumber(6.022e23));
        assert_eq!(tail, ",");
    }

    #[test]
    fn test_parse_number_exponent_negative() {
        let input = "1e-10]";
        let (val, tail) = parse_number(input).unwrap();
        assert_eq!(val, Json::JsonNumber(1e-10));
        assert_eq!(tail, "]");
    }

    #[test]
    fn test_parse_number_exponent_positive_sign() {
        let input = "2E+2 ";
        let (val, tail) = parse_number(input).unwrap();
        assert_eq!(val, Json::JsonNumber(200.0));
        assert_eq!(tail, " ");
    }

    #[test]
    fn test_parse_number_only_minus() {
        let input = "- rest";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_letter() {
        let input = "a0123";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_leading_zero() {
        let input = "0123";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_leading_exponent() {
        let input = "e0123";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_leading_dot() {
        let input = ".123";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_double_dot() {
        let input = "1..2";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_exponent() {
        let input = "1e";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_inf() {
        let input = "inf";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_ninf() {
        let input = "ninf";
        assert!(parse_number(input).is_err());
    }

    #[test]
    fn test_parse_number_invalid_nan() {
        let input = "NaN";
        assert!(parse_number(input).is_err());
    }
}