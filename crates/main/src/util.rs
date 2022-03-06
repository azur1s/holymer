use std::fmt::Display;

pub fn log<T: Display>(level: i8, msg: T) {
    match level {
        0 => println!("\x1b[92m[INFO]\x1b[0m {}", msg),
        1 => println!("\x1b[93m[WARN]\x1b[0m {}", msg),
        2 => println!("\x1b[91m[ERRS]\x1b[0m {}", msg),
        _ => println!("{}", msg),
    }
}