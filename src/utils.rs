fn is_ws(c: char) -> bool {
    c == '\u{0020}' || c == '\u{000A}' || c == '\u{000D}' || c == '\u{0009}'
}

pub fn ignore_ws(source: &str) -> &str {
    let mut s = source;
    while let Some(c) = s.chars().next() {
        if is_ws(c) {
            s = &s[c.len_utf8()..];
        } else {
            break;
        }
    }
    s
}