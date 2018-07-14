
use simplelog::*;

pub fn configure_logging(log_level: u32) {
    let level = match log_level {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => LevelFilter::Trace,
    };
    ::simplelog::TermLogger::init(level, ::simplelog::Config::default()).unwrap();
}
