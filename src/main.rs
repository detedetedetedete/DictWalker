extern crate serde_json;
extern crate encoding;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate fern;
extern crate clap;
extern crate chrono;

use dict_entry::DictEntry;

use std::path::Path;
use cli_api::get_args;
use logging::setup_logger;
use logging::level_from_string;


mod decode;
mod dict_entry;
mod cli_api;
mod logging;

fn main() {
    let matches = get_args();
    setup_logger(
        level_from_string(
            matches.value_of("log level").unwrap()
        ).unwrap()
    ).expect("Failed to initialize logging.");

    let dictionary = matches.value_of("dictionary").unwrap();

    let entries =  match DictEntry::collect_entries(Path::new(dictionary)) {
        Ok(v) => v,
        Err(e) => panic!("Failed to collect entries: {:?}", e)
    };
    // TODO: perform some postprocessing one the transcript
    // TODO: transform transcript to graphemes
    info!("Entries: {:#?}", entries);

    let json = match serde_json::to_string_pretty(&entries) {
        Ok(v) => v,
        Err(e) => panic!(e)
    };

    info!("Entry json: {}", json);
}
