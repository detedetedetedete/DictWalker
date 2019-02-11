use clap::App;
use clap::Arg;
use clap::ArgMatches;
use std::path::Path;
use logging::level_from_string;
use regex::Regex;
use logging::setup_logger;

fn valid_s2s_model(path_str: String) -> Result<(), String> {
    valid_dir(path_str.clone())?;
    valid_input_file(format!("{}/model.json", path_str))?;
    valid_input_file(format!("{}/encoder_inference_model.pb", path_str))?;
    valid_input_file(format!("{}/decoder_inference_model.pb", path_str))?;
    Ok(())
}

fn valid_dir(path_str: String) -> Result<(), String> {
    let path = Path::new(&path_str);
    if path.is_dir() {
        Ok(())
    } else {
        Err(format!("Path \"{}\" does not exist or is not a directory.", path_str))
    }
}

fn valid_dest_file(path_str: String) -> Result<(), String> {
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent()  {
        if parent.exists() {
            Ok(())
        } else {
            Err(format!("Directory to file \"{}\" does not exist.", path_str))
        }

    } else {
        Err(format!("Cannot resolve parent directory of \"{}\".", path_str))
    }
}

fn valid_input_file(path_str: String) -> Result<(), String> {
    let path = Path::new(&path_str);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("Path {:?} does not exist or is not a file.", path))
    }
}

fn valid_level(log_level_str: String) -> Result<(), String> {
    match level_from_string(log_level_str.as_ref()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

fn extension_list_valid(extension: String) -> Result<(), String> {
    lazy_static! {
        static ref ext_list_re: Regex = Regex::new(r"[a-zA-Z0-9,]").unwrap();
    }
    match ext_list_re.is_match(extension.as_ref()) {
        true => Ok(()),
        false => Err(format!("\"{}\" is not a valid extension list.", extension))
    }
}

pub fn get_args() -> ArgMatches<'static> {
    let matches = App::new("Audio dictionary walker")
        .version("0.1.0")
        .author("DT <deltakowsz@gmail.com>")
        .about("Walks the text <-> audio dictionary and produces a JSON with phonemes")
        .arg(
            Arg::with_name("dictionary")
                .short("i")
                .long("dictionary")
                .value_name("DIRECTORY")
                .help("Path to the text <-> audio dictionary")
                .takes_value(true)
                .required(true)
                .validator(valid_dir)
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Path to output json file")
                .takes_value(true)
                .required(true)
                .validator(valid_dest_file)
                .default_value("./output.json")
        )
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .help("Allow overwriting of the output file")
                .takes_value(false)
                .required(false)
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
        .arg(
            Arg::with_name("audio extensions")
                .short("a")
                .long("audio")
                .value_name("EXTS")
                .help("comma delimited audio extensions")
                .takes_value(true)
                .required(false)
                .validator(extension_list_valid)
                .default_value("wav")
        )
        .arg(
            Arg::with_name("text extensions")
                .short("t")
                .long("text")
                .value_name("EXTS")
                .help("comma delimited text extensions")
                .takes_value(true)
                .required(false)
                .validator(extension_list_valid)
                .default_value("txt")
        )
        .arg(
            Arg::with_name("phoneme dictionary")
                .short("p")
                .long("phonemes")
                .value_name("DICTIONARY")
                .help("path to grapheme-phoneme dictionary")
                .takes_value(true)
                .required(false)
                .validator(valid_input_file)
        )
        .arg(
            Arg::with_name("Seq2Seq model folder")
                .short("m")
                .long("model")
                .value_name("FOLDER")
                .help("Seq2Seq grapheme to phoneme model folder")
                .takes_value(true)
                .required(false)
                .validator(valid_s2s_model)
        )
        .get_matches();


    setup_logger(
        level_from_string(
            matches.value_of("log level").unwrap()
        ).unwrap()
    ).expect("Failed to initialize logging.");

    if !matches.is_present("force") {
        let path = Path::new(matches.value_of("output").unwrap());
        if path.exists() {
            error!("Output file already exists! Use the --force (or -f) to force overwriting of the output file.");
            panic!();
        }

    }

    matches
}