mod core;
mod utils;

mod object;
mod array;
mod string;
mod literals;
mod number;

pub use core::{Json, JsonError, parse};

impl From<&str> for Json {
    fn from(val: &str) -> Json {
        let result : Result<(Json, &str), JsonError> = parse(val);
        if let Ok((json, _)) = result {
            json
        } else {
            Json::JsonEmpty
        }
    }
}

impl From<String> for Json {
    fn from(val: String) -> Json {
        Json::from(val.as_str())
    }
}
