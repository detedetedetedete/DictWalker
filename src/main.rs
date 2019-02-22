extern crate serde_json;
extern crate encoding;
extern crate fern;
extern crate clap;
extern crate chrono;
extern crate regex;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate serde;
extern crate libc;

use dict_entry::DictEntry;

use std::path::Path;
use cli_api::get_args;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::fs::File;
use std::io::Write;
use training_entry::TrainingEntry;
use phoneme_resolvers::DeadEndPhonemeResolver;
use phoneme_resolvers::DictionaryPhonemeResolver;
use phoneme_resolvers::PhonemeResolver;
use phoneme_resolvers::MarkerPhonemeResolver;
use phoneme_resolvers::DummyPhonemeResolver;
use phoneme_resolvers::TensorflowPhonemeResolver;


mod decode;
mod dict_entry;
mod cli_api;
mod logging;
mod training_entry;
mod phonemes;
mod phoneme_resolvers;
mod model_def;
mod model_runner;

fn main() {
    let matches = get_args();
    let dictionary = matches.value_of("dictionary").unwrap();
    let mut output_file = match File::create(matches.value_of("output").unwrap()) {
        Ok(v) => v,
        Err(e) => {
            error!("Cannot open file \"{}\" for writing: {}", matches.value_of("output").unwrap(), e);
            panic!();
        }
    };

    //TODO: add type for end product (transcript, phonemes, path to audio)
    //TODO: add path to grapheme to phoneme dictionary (not mandatory)
    //TODO: add path to grapheme to phoneme converter NN (not mandatory)
    //TODO: transform transcript to phonemes
    //TODO: clean unwraps

    let mut phoneme_resolvers: Vec<Box<PhonemeResolver>> = vec![
        match matches.value_of("phoneme dictionary") {
            Some(path) => {
                match DictionaryPhonemeResolver::load(Path::new(path)) {
                    Ok(v) => Box::new(v),
                    Err(e) => {
                        error!("Failed to instantiate DictionaryPhonemeResolver: \"{}\"", e);
                        panic!();
                    }
                }
            },
            None => Box::new(DummyPhonemeResolver::new())
        },
        match matches.value_of("Seq2Seq model folder") {
            Some(path) => {
                let model_folder_path = Path::new(path);
                match TensorflowPhonemeResolver::load(model_folder_path) {
                    Ok(v) => Box::new(v),
                    Err(e) => {
                        error!("Failed to instantiate DictionaryPhonemeResolver: \"{}\"", e);
                        panic!();
                    }
                }
            },
            None => Box::new(DummyPhonemeResolver::new())
        },
        Box::new(MarkerPhonemeResolver::new()),
        Box::new(DeadEndPhonemeResolver::new())
    ];

    let mut entries =  match DictEntry::collect_entries(
        Path::new(dictionary),
        &HashSet::from_iter(matches.value_of("audio extensions").unwrap().split(",").map(|v| String::from(v))),
        &HashSet::from_iter(matches.value_of("text extensions").unwrap().split(",").map(|v| String::from(v)))
    ) {
        Ok(v) => v,
        Err(e) => panic!("Failed to collect entries: {:?}", e)
    };

    let t_entries: Vec<TrainingEntry> = entries
        .drain(0..)
        .map(|v| TrainingEntry::construct(v, &mut phoneme_resolvers))
        .collect();

    //error!("{:#?}", t_entries);

    let json = match serde_json::to_string_pretty(&t_entries) {
        Ok(v) => v,
        Err(e) => {
            error!("Cannot serialize processed entries to JSON: {}", e);
            panic!();
        }
    };

    match output_file.write_all(json.as_bytes()) {
        Err(e) => {
            error!("Error during write to file {:?}: {}", output_file, e);
            panic!();
        },
        _ => ()
    };

    info!("Done.");
}
