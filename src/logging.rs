use chrono;
use fern;
use log::info;

pub(crate) fn setup_logging(level: log::LevelFilter) {
    match fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
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

