use ansi_term::Color::*;
use log::{Level, SetLoggerError};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &log::Record) {
        let level_colored = match record.level() {
            Level::Error => Red.paint(record.level().as_str()),
            Level::Warn => Yellow.paint(record.level().as_str()),
            Level::Info => Blue.paint(record.level().as_str()),
            Level::Debug => Green.paint(record.level().as_str()),
            Level::Trace => Black.on(Yellow).bold().paint(record.level().as_str()),
        };

        if self.enabled(record.metadata()) {
            println!(
                "{}:{}: {} - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                level_colored,
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| 
        log::set_max_level(log::LevelFilter::Trace)
    )
}
