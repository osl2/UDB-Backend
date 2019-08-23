use chrono;
use fern;
use log::info;

pub(crate) fn setup_logging(level: log::LevelFilter) {
    match fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or_else(|| "unknown_module"),
                record.file().unwrap_or_else(|| "unknown_file"),
                record.line().unwrap_or_else(|| 0u32),
                message,
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
    {
        Err(_) => {
            eprintln!("error setting up logging!");
        }
        _ => info!("logging set up properly"),
    }
}
