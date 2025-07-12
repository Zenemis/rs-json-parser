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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let (json, tail) = parse(r#"{"a":1,"b":"str"}"#).unwrap();
        assert_eq!(tail, "");
        if let Json::JsonObject(map) = json {
            assert_eq!(map["a"], Json::JsonNumber(1.0));
            assert_eq!(map["b"], Json::JsonString("str".to_string()));
        } else {
            panic!("Expected JsonObject");
        }
    }

    #[test]
    fn test_parse_simple_array() {
        let (json, tail) = parse(r#"[1, "two", null]"#).unwrap();
        assert_eq!(tail, "");
        if let Json::JsonArray(arr) = json {
            assert_eq!(arr[0], Json::JsonNumber(1.0));
            assert_eq!(arr[1], Json::JsonString("two".to_string()));
            assert_eq!(arr[2], Json::JsonNull);
        } else {
            panic!("Expected JsonArray");
        }
    }

    #[test]
    fn test_parse_nested_object_array() {
        let (json, tail) = parse(r#"{"arr":[1,2,3],"obj":{"k":false}}"#).unwrap();
        assert_eq!(tail, "");
        if let Json::JsonObject(map) = json {
            if let Json::JsonArray(arr) = &map["arr"] {
                assert_eq!(arr[1], Json::JsonNumber(2.0));
            } else {
                panic!("Expected JsonArray for 'arr'");
            }
            if let Json::JsonObject(obj) = &map["obj"] {
                assert_eq!(obj["k"], Json::JsonBoolean(false));
            } else {
                panic!("Expected JsonObject for 'obj'");
            }
        } else {
            panic!("Expected JsonObject");
        }
    }

    #[test]
    fn test_parse_invalid_json() {
        assert!(parse("{").is_err());
        let (json, tail) = parse("   [ 1 , 2 , 3 ]   ").unwrap();
        assert_eq!(
            json,
            Json::JsonArray(vec![
                Json::JsonNumber(1.0),
                Json::JsonNumber(2.0),
                Json::JsonNumber(3.0)
            ])
        );
        assert_eq!(tail.trim(), "");
        assert!(parse("nulll").is_err());
    }

    #[test]
    fn test_parse_complex_nested_object_and_array() {
        let src = r#"
        {
            "name": "Alice",
            "age": 30,
            "is_active": true,
            "scores": [10, 20, 30.5, null],
            "address": {
                "city": "Wonderland",
                "zip": "12345"
            },
            "tags": [],
            "meta": {
                "created": null,
                "roles": ["admin", "user"]
            }
        }
        "#;
        let (json, tail) = parse(src).unwrap();
        assert_eq!(tail.trim(), "");
        if let Json::JsonObject(map) = json {
            assert_eq!(map["name"], Json::JsonString("Alice".to_string()));
            assert_eq!(map["age"], Json::JsonNumber(30.0));
            assert_eq!(map["is_active"], Json::JsonBoolean(true));
            if let Json::JsonArray(scores) = &map["scores"] {
                assert_eq!(scores[0], Json::JsonNumber(10.0));
                assert_eq!(scores[1], Json::JsonNumber(20.0));
                assert_eq!(scores[2], Json::JsonNumber(30.5));
                assert_eq!(scores[3], Json::JsonNull);
            } else {
                panic!("Expected scores to be array");
            }
            if let Json::JsonObject(addr) = &map["address"] {
                assert_eq!(addr["city"], Json::JsonString("Wonderland".to_string()));
                assert_eq!(addr["zip"], Json::JsonString("12345".to_string()));
            } else {
                panic!("Expected address to be object");
            }
            if let Json::JsonArray(tags) = &map["tags"] {
                assert!(tags.is_empty());
            } else {
                panic!("Expected tags to be array");
            }
            if let Json::JsonObject(meta) = &map["meta"] {
                assert_eq!(meta["created"], Json::JsonNull);
                if let Json::JsonArray(roles) = &meta["roles"] {
                    assert_eq!(roles[0], Json::JsonString("admin".to_string()));
                    assert_eq!(roles[1], Json::JsonString("user".to_string()));
                } else {
                    panic!("Expected roles to be array");
                }
            } else {
                panic!("Expected meta to be object");
            }
        } else {
            panic!("Expected JsonObject");
        }
    }

    #[test]
    fn test_parse_array_of_objects_and_arrays() {
        let src = r#"
        [
            {"id": 1, "values": [1,2,3]},
            {"id": 2, "values": []},
            [],
            [null, false, {"nested": [42]}]
        ]
        "#;
        let (json, tail) = parse(src).unwrap();
        assert_eq!(tail.trim(), "");
        if let Json::JsonArray(arr) = json {
            if let Json::JsonObject(obj1) = &arr[0] {
                assert_eq!(obj1["id"], Json::JsonNumber(1.0));
                if let Json::JsonArray(vals) = &obj1["values"] {
                    assert_eq!(vals[0], Json::JsonNumber(1.0));
                    assert_eq!(vals[1], Json::JsonNumber(2.0));
                    assert_eq!(vals[2], Json::JsonNumber(3.0));
                } else {
                    panic!("Expected values to be array");
                }
            } else {
                panic!("Expected first element to be object");
            }
            if let Json::JsonObject(obj2) = &arr[1] {
                assert_eq!(obj2["id"], Json::JsonNumber(2.0));
                if let Json::JsonArray(vals) = &obj2["values"] {
                    assert!(vals.is_empty());
                } else {
                    panic!("Expected values to be array");
                }
            }
            if let Json::JsonArray(empty) = &arr[2] {
                assert!(empty.is_empty());
            }
            if let Json::JsonArray(mixed) = &arr[3] {
                assert_eq!(mixed[0], Json::JsonNull);
                assert_eq!(mixed[1], Json::JsonBoolean(false));
                if let Json::JsonObject(nested) = &mixed[2] {
                    if let Json::JsonArray(nested_arr) = &nested["nested"] {
                        assert_eq!(nested_arr[0], Json::JsonNumber(42.0));
                    } else {
                        panic!("Expected nested to be array");
                    }
                } else {
                    panic!("Expected nested to be object");
                }
            }
        } else {
            panic!("Expected JsonArray");
        }
    }
}

