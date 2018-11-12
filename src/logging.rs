use log::LevelFilter;
use fern::colors::ColoredLevelConfig;
use fern::colors::Color;

pub fn setup_logger(level: LevelFilter) -> Result<(), fern::InitError> {

    let colors = ColoredLevelConfig::new()
        .trace(Color::White)
        .debug(Color::Yellow)
        .info(Color::Green)
        .warn(Color::BrightMagenta)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

pub fn level_from_string(level_str: &str) -> Result<LevelFilter, String> {
    match level_str.to_lowercase().as_ref() {
        "trace" => Ok(LevelFilter::Trace),
        "debug" => Ok(LevelFilter::Debug),
        "info" => Ok(LevelFilter::Info),
        "warn" => Ok(LevelFilter::Warn),
        "error" => Ok(LevelFilter::Error),
        _ => Err(format!("\"{}\" is not a valid LevelFilter value.", level_str))
    }
}