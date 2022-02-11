// This implementation of parser is heavily inspired by
// brundonsmith/rust_lisp (https://github.com/brundonsmith/rust_lisp)
// go check them out!

use std::{ rc::Rc, fmt };

#[derive(Debug, Clone)]
pub enum Value {
    // Boolean types
    True, False,
    // Numbers
    Int(i64), Float(f64),

    String(String), Symbol(String),    
    List(Rc<Value>, Rc<Vec<Value>>),

    Nil,
}

#[derive(Debug, Clone)]
pub enum Tree {
    Atom { atom: Value, quote: bool },
    List { vec: Vec<Tree>, quote: bool },
}

impl Tree {
    fn into_expr(self) -> Value {
        match self {
            Tree::Atom { atom, quote } => {
                if quote {
                    Value::List(
                        Rc::new(Value::Symbol(String::from("quote"))),
                        Rc::new(vec![atom])
                    )
                } else {
                    atom
                }
            },
            Tree::List { vec, quote } => {
                let list = Value::List(
                    Rc::new(vec[0].clone().into_expr()),
                    Rc::new(vec[1..].iter().map(|a| a.clone().into_expr()).collect())
                );

                if quote {
                    Value::List(
                        Rc::new(Value::Symbol(String::from("quote"))),
                        Rc::new(vec![list])
                    )
                } else {
                    list
                }
            }
        }
    }
}

// --- Start tokenizer ---

const SPECIAL_CHARS: [&str; 4] = ["(", ")", "'", "..."];

/// Match the characters from `with` with the characters from `from`
/// Example: match_front("123", "12") -> true
fn match_front(from: &str, with: &str) -> bool { with.chars().zip(from.chars()).all(|(a, b)| a == b) }

/// Get length from `from` until `predicate`
/// Example: match_pred("abcdef", |c| c != 'f') -> Some(5)
fn match_predicate<F: Fn(char) -> bool>(from: &str, predicate: F) -> Option<usize> {
    from.char_indices().take_while(|(_, c)| predicate(*c)).last().map(|(i, _)| i)
}

/// Check if characters is in a special characters list or not
fn is_symbolic(char: char) -> bool {
    !char.is_whitespace() && !SPECIAL_CHARS.iter().any(|t| t.chars().any(|other| other == char))
}

/// Return type: (token, (start, end))
pub fn tokenize(src: &str) -> impl Iterator<Item = (&str, (usize, usize))> {
    let mut skip: Option<usize> = None;

    src.char_indices().filter_map(move |(i, char)| {

        if skip.map(|dest| dest > i).unwrap_or(false) { return None; }
        else { skip = None; }
   
        // Whitespaces
        if char.is_whitespace() { return None; }

        // Special characters
        for special in &SPECIAL_CHARS {
            if match_front(&src[i..], special) {
                skip = Some(i + special.len());
                return Some((*special, (i, i + special.len())));
            }
        }
        
        // Strings
        if char == '"' {
            let match_end = match_predicate(&src[i + 1..], |c| c != '"');
            
            if let Some(end) = match_end {
                let string_end = i + end + 3;
                skip = Some(string_end);
                return Some((&src[i..string_end], (i, string_end)));
            }
        }

        // Comments
        // Check if the current char is a semicolon and  
        if char == ';' && src[i + 1..].chars().next().map_or(false, |c| c == ';') {
            // Get length until end of line
            let end = i + 2 + match_predicate(&src[i + 2..], |c| c!= '\n').unwrap_or(0);

            skip = Some(end + 1);
            return None;
        }

        // Numbers
        if char.is_numeric() {
            let front = i + match_predicate(&src[i..], |c| c.is_numeric()).unwrap() + 1;

            // Check if its a float (by checking if its contain a dot)
            if front < src.len() - 1 && &src[front..front + 1] == "." {
                let back = front + match_predicate(&src[front + 1..], |c| c.is_numeric()).unwrap() + 2;
                skip = Some(back);
                return Some((&src[i..back], (i, back)));
            } else {
                skip = Some(front);
                return Some((&src[i..front], (i, front)));
            }
        }

        // Symbols
        if !char.is_numeric() && is_symbolic(char) {
            let end = match_predicate(&src[i..], is_symbolic);

            if let Some(last) = end {
                let symbol_end = i + last + 1;

                skip = Some(symbol_end);
                return Some((&src[i..symbol_end], (i, symbol_end)));
            }
        }

        None
    })
}

// --- End tokenizer & Start parser ---

#[derive(Debug)]
pub enum ParseErrorKind {
    UnexpectedParenClose,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErrorKind::UnexpectedParenClose => write!(f, "Unexpected ')'"),
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub pos: (usize, usize),
}

impl ParseError {
    fn new(kind: ParseErrorKind, pos: (usize, usize)) -> Self {
        ParseError { kind, pos }
    }

    pub fn at(&self, src: &str) -> String {
        let snip = &src[(self.pos.0.saturating_sub(5))..(if self.pos.0 + 5 > src.len() { src.len() } else { self.pos.0 + 5 })];
        format!("\n{}..{}\n{}\nError: {} at {}", " ".repeat(3), snip, format!("{}^", " ".repeat(10)), self.kind, self.pos.0)
        
        // Example:
        //
        //   .."))) ) (pr
        //          ^
        // Error: Unexpected ')' at 67
    }
}

fn read<'a>(
    tokens: impl Iterator<Item = (&'a str, (usize, usize))> + 'a
    )    -> impl Iterator<Item = Result<(Value, (usize, usize)), ParseError>> + 'a {
    let mut stack: Vec<Tree> = Vec::new();
    let mut parenths = 0;
    let mut quote_next = false;

    let mut block_start = 0;

    tokens.filter_map(move |(token, (start, end))| {
        match token {
            "(" => {
                parenths += 1;
                
                if parenths == 1 {
                    block_start = start;
                }

                stack.push(Tree::List {
                    vec: Vec::new(),
                    quote: quote_next,
                });
                quote_next = false;

                None
            },
            ")" => {
                parenths -= 1;
                
                if stack.is_empty() {
                    Some(Err(ParseError::new(
                        ParseErrorKind::UnexpectedParenClose,
                        (start, end)
                    )))
                } else {
                    let mut finished = stack.pop().unwrap();

                    if parenths == 0 {
                        stack = Vec::new();
                        let r = Some(Ok((finished.into_expr(), (block_start, end))));
                        block_start = 0;
                        r
                    } else {
                        let destination = stack.last_mut().unwrap();

                        if let Tree::List { vec, quote } = &finished {
                            if vec.is_empty() {
                                finished = Tree::Atom {
                                    atom: Value::Nil,
                                    quote: *quote,
                                };
                            }
                        }

                        if let Tree::List { vec, quote: _ } = destination { vec.push(finished); }

                        None
                    }
                }
            },
            "'" => { quote_next = true; None },
            _ => {
                let expr = Tree::Atom {
                    atom: read_atom(token),
                    quote: quote_next,
                };
                quote_next = false;

                if let Some(last) = stack.last_mut() {
                    if let Tree::List { vec, quote: _ } = last {
                        vec.push(expr);
                    }
                    None
                } else {
                    Some(Ok((expr.into_expr(), (start, end))))
                }
            }
        }
    })
}

fn read_atom(token: &str) -> Value {
    let lower = token.to_lowercase();

    match lower.as_str() {
        "true" => Value::True,
        "false" => Value::False,
        "nil" => Value::Nil,
        _ => {
            // Parse number
            if let Ok(int) = token.parse::<i64>() { Value::Int(int) }
            // Parse float
            else if let Ok(float) = token.parse::<f64>() { Value::Float(float) }
            // Parse string
            else if token.chars().next().map_or(false, |c| c == '"') && token.chars().nth_back(0).map_or(false, |c| c == '"') {
                Value::String(String::from(&token[1..token.chars().count() - 1]))
            } else {
                Value::Symbol(String::from(token))
            }
        }
    }
}

// --- End parser ---

pub fn parse(src: &str) -> impl Iterator<Item = Result<(Value, (usize, usize)), ParseError>> + '_ {
    read(tokenize(src))
}
