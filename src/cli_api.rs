use clap::App;
use clap::Arg;
use clap::ArgMatches;
use std::path::Path;
use logging::level_from_string;

fn valid_dir(path_str: String) -> Result<(), String> {
    let path = Path::new(&path_str);
    if path.exists() && path.is_dir() {
        Ok(())
    } else {
        Err(format!("Path \"{}\" does not exist or is not a directory.", path_str))
    }
}

fn valid_level(log_level_str: String) -> Result<(), String> {
    match level_from_string(log_level_str.as_ref()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

pub fn get_args() -> ArgMatches<'static> {
    App::new("Audio dictionary walker")
        .version("0.1.0")
        .author("DT <deltakowsz@gmail.com>")
        .about("Walks the text <-> audio dictionary and produces a JSON with graphemes")
        .arg(
            Arg::with_name("dictionary")
                .short("d")
                .long("dictionary")
                .value_name("DIRECTORY")
                .help("Path to the text <-> audio dictionary")
                .takes_value(true)
                .required(true)
                .validator(valid_dir)
        )
        .arg(
            Arg::with_name("log level")
                .short("l")
                .long("level")
                .value_name("LEVEL")
                .help("logging level")
                .takes_value(true)
                .required(false)
                .validator(valid_level)
                .default_value("info")
        )
        .get_matches()
    //TODO: add parameters for supported audio and text extensions
    //TODO: add path to grapheme to phoneme dictionary (not mandatory)
    //TODO: add path to grapheme to phoneme converter NN (not mandatory)
}