use std::str::CharIndices;

use crate::core;

use core::{Json, JsonError};

fn parse_utf8_hex(chars: &mut CharIndices) -> Option<char> {
    let mut unicode = String::new();
    for _ in 0..4 {
        if let Some((_, uc)) = chars.next() {
            unicode.push(uc);
        } else {
            return None;
        }
    }
    let code = match u16::from_str_radix(&unicode, 16) {
        Ok(c) => c,
        Err(_) => return None,
    };
    std::char::from_u32(code as u32)
}

fn parse_escape_char(c: char) -> Option<char> {
    match c {
        '"'  => Some('"'),
        '\\' => Some('\\'),
        '/'  => Some('/'),
        'b'  => Some('\u{0008}'),
        'f'  => Some('\u{000C}'),
        'n'  => Some('\n'),
        'r'  => Some('\r'),
        't'  => Some('\t'),
        _    => None,
    }
}

pub fn parse_string(source: &str) -> Result<(Json, &str), JsonError> {
    let mut chars: CharIndices<'_> = source.char_indices();

    if let Some((_, first)) = chars.next() {
        if first != '"' {
            return Err(JsonError::SyntaxError(format!("Expected '\"' at start of string, found '{}'", first)));
        }
    } else {
        return Err(JsonError::SyntaxError("Unexpected end of input while parsing string".to_string()));
    }

    let mut result = String::new();
    let mut escape = false;

    while let Some((i, c)) = chars.next() {
        if escape {
            result.push(if c == 'u' {
                parse_utf8_hex(&mut chars).ok_or(JsonError::LexicalError(format!("Invalid escape character in utf-8 hex : {}", c)))?
            } else if let Some(unescaped) = parse_escape_char(c) {
                unescaped
            } else {
                return Err(JsonError::LexicalError(format!("Invalid escape character : {}", c)));
            });
            escape = false;
        } else {
            if c == '"' {
                // End of string
                let tail = &source[i + 1..];
                return Ok((Json::JsonString(result), tail));
            } else if c == '\\' {
                escape = true;
            } else {
                result.push(c);
            }
        }
    }
    Err(JsonError::SyntaxError(format!("Invalid string : \"{}", source)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unicode_valid_ascii() {
        let mut iter = "0041".char_indices(); // 'A'
        assert_eq!(parse_utf8_hex(&mut iter), Some('A'));
        assert_eq!(iter.next(), None); // Iterator should be at the end
    }

    #[test]
    fn test_parse_utf8_hex_valid_bmp() {
        let mut iter = "03A9tail".char_indices(); // 'Ω'
        assert_eq!(parse_utf8_hex(&mut iter), Some('Ω'));
        assert_eq!(iter.next(), Some((4, 't'))); // Iterator should have progressed
    }

    #[test]
    fn test_parse_utf8_hex_valid_bmp_lowercase() {
        let mut iter = "03a9tail".char_indices(); // 'Ω'
        assert_eq!(parse_utf8_hex(&mut iter), Some('Ω'));
        assert_eq!(iter.next(), Some((4, 't'))); // Iterator should have progressed
    }

    #[test]
    fn test_parse_utf8_hex_invalid_empty() {
        let mut iter = "".char_indices(); // Empty
        assert_eq!(parse_utf8_hex(&mut iter), None);
    }

    #[test]
    fn test_parse_utf8_hex_invalid_short() {
        let mut iter = "41".char_indices(); // Too short
        assert_eq!(parse_utf8_hex(&mut iter), None);
    }

    #[test]
    fn test_parse_utf8_hex_invalid_nonhex() {
        let mut iter = "ZZZZ".char_indices(); // Not hex
        assert_eq!(parse_utf8_hex(&mut iter), None);
    }

    #[test]
    fn test_parse_string_valid() {
        let valid = "\"field\"   ";
        let (field, tail) = parse_string(valid).unwrap();
        assert_eq!(field, Json::JsonString("field".to_string()));
        assert_eq!(tail, "   ");
    }

    #[test]
    fn test_parse_string_valid_escape_characters() {
        let valid = "\"line\\nbreak\\tand\\tescape\\\\quote\\\"end\" rest";
        let (parsed, tail) = parse_string(valid).unwrap();
        assert_eq!(
            parsed,
            Json::JsonString("line\nbreak\tand\tescape\\quote\"end".to_string())
        );
        assert_eq!(tail, " rest");
    }

    #[test]
    fn test_parse_string_valid_utf8_hex() {
        let valid = "\"Omega: \\u03A9, A: \\u0041, smile: \\u263A!\" next";
        let (parsed, tail) = parse_string(valid).unwrap();
        assert_eq!(
            parsed,
            Json::JsonString("Omega: Ω, A: A, smile: ☺!".to_string())
        );
        assert_eq!(tail, " next");
    }

    #[test]
    fn test_parse_string_valid_empty() {
        let valid = "\"\"";
        let (field, tail) = parse_string(valid).unwrap();
        assert_eq!(field, Json::JsonString("".to_string()));
        assert_eq!(tail, "");
    }

    #[test]
    fn test_parse_string_invalid_empty() {
        let invalid = "";
        assert!(parse_string(invalid).is_err());
    }

    #[test]
    fn test_parse_string_invalid() {
        // Unterminated string
        let invalid = "\"abc\\u0041";
        assert!(parse_string(invalid).is_err());

        // Invalid escape
        let invalid2 = "\"bad\\xescape\"";
        assert!(parse_string(invalid2).is_err());

        // Invalid unicode
        let invalid3 = "\"bad\\uZZZZ\"";
        assert!(parse_string(invalid3).is_err());
    }

}