pub fn cover_paren(s: String) -> String {
    format!("({})", s)
}

pub fn unescape(s: String) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < s.len() {
        if s.chars().nth(i).unwrap() == '\\' {
            match s.chars().nth(i + 1).unwrap() {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                _ => result.push(s.chars().nth(i + 1).unwrap()),
            }
            i += 2;
        } else {
            result.push(s.chars().nth(i).unwrap());
            i += 1;
        }
    }
    result
}