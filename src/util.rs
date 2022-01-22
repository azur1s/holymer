use crate::token::Expr::{self, List};

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

pub fn unwrap_list_nest(ast: Expr) -> Vec<Expr> {
    let mut result: Vec<Expr> = Vec::new();

    match ast.clone() {
        List(l, _) => {
            for expr in l.iter() {
                
                result.push(expr.clone());

            }
        }
        _ => {
            // This probably will not happen because everything is wrapped
            // in list. So it would be impossible that the ast is not a list.
            eprintln!("Possibly a bug in the compiler, you shouln't get this message.");
            dbg!(ast);
        }
    };

    result
}