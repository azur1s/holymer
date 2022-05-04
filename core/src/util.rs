use colored::Colorize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel { Debug, Info, Warn, Error }

fn prefix (level: LogLevel) -> String {
    match level {
        LogLevel::Debug => "DEBG ".bright_black(),
        LogLevel::Info  => "INFO ".blue(),
        LogLevel::Warn  => "WARN ".yellow(),
        LogLevel::Error => "ERRO ".red(),
    }.to_string()
}

pub fn log <S: Into<String>>(level: LogLevel, message: S) {
    println!("{}{}", prefix(level), message.into());
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::util::log( $crate::util::LogLevel::Info, format!($($arg)*) );
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::util::log( $crate::util::LogLevel::Warn, format!($($arg)*) );
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::util::log( $crate::util::LogLevel::Error, format!($($arg)*) );
    };
}