use clap::{Arg, ArgAction, Command};

pub(crate) fn setup_cli() -> clap::ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("v")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count)
                .help("Be verbose (you can add this up to 4 times for more logs)"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Set config file path"),
        )
        .get_matches()
}
