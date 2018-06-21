
use ::simplelog::*;

pub fn configure_logging(log_level: u32) {
    let level = match log_level {
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => LevelFilter::Error,
    };
    ::simplelog::TermLogger::init(level, ::simplelog::Config::default()).unwrap();
}