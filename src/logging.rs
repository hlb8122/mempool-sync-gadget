use chrono::Utc;
use colored::*;
use flexi_logger::{DeferredNow, Logger};
use log::{Level, Record};

fn format_log(
    w: &mut std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}, {}] - {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S.%f").to_string().blue(),
        match record.level() {
            Level::Info => "INFO".green(),
            Level::Warn => "WARN".yellow(),
            Level::Error => "ERROR".red(),
            Level::Debug => "DEBUG".magenta(),
            Level::Trace => "TRACE".cyan(),
        },
        record.args()
    )
}

pub fn init_console_logging() {
    flexi_logger::Logger::with_env()
        .format(format_log)
        .start()
        .unwrap();
}

pub fn init_file_logging() {
    Logger::with_str("mempool_sync_gadget = info")
        .format(format_log)
        .log_to_file()
        .start()
        .unwrap();
}
