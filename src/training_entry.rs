use phonemes::Phoneme;
use dict_entry::DictEntry;
use regex::Regex;
use phoneme_resolvers::PhonemeResolver;
use std::str::FromStr;
use serde::Serializer;

#[derive(Debug, Serialize)]
pub struct TrainingEntry {
    pub transcript: String,
    #[serde(serialize_with = "serialize_phoneme_vec")]
    pub phonemes: Vec<Phoneme>,
    pub audio_path: String
}

fn serialize_phoneme_vec<S>(vec: &Vec<Phoneme>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut str = String::new();
    for ph in vec {
        str.push_str(&ph.to_string());
    }
    serializer.serialize_str(&str)
}

impl TrainingEntry {
    //TODO: perform some postprocessing on the transcript
    // --encoding errors:
    // remove \u{feff}
    // replace \u{9a} with ž
    // remove \u{1f}

    // --errors:
    // _centrai centrai - error in transcript, included twice
    // indais _dais - fix detached accent
    // _is kvepimas - fix misspelled marker
    // _puslpais - spelling error, fix to _puslapis
    // _dutys - spelling error, fix to _durys

    // rename markers:
    // _pauze
    // _tyla
    // _ikvepimas
    // _iskvepimas
    // _nurijimas
    // _cepsejimas
    // _kede
    // _pilvas
    // _garsas
    // _puslapis
    // _durys
    // _eh
    // - //middle word pauses

    // --accents:
    // _ prefix
    // _[A-Za-z] postfix

    // --post cleaning ops:
    // remove \r \n and \t
    // remove spaces from start of string
    // remove trailing spaces
    // replace multiple spaces with a single space

    fn fix_encoding_errors(str: String) -> String {
        str.replace("\u{feff}", "")
            .replace("\u{9a}", "ž")
            .replace("\u{1f}", "")
    }

    fn fix_spelling_errors(str: String) -> String {
        str.replace("_centrai centrai", "centrai")
            .replace("indais _dais", "indais_dais")
            .replace("_is kvepimas", "_iskvepimas")
            .replace("_puslpais", "_puslapis")
            .replace("_dutys", "_durys")
            .replace("Simono-Petro", "Simono Petro")
            .replace("Achemenidu", "Achemenidų")
            //.replace("_ikvepimasvykdyk", "_ikvepimas vykdyk")
            //.replace("laiškas_ikvepimas", "laiškas _ikvepimas")
            //.replace("mastelį_pauze", "mastelį _pauze")
            //.replace("mastelį_tyla", "mastelį _tyla")
    }

    fn process_markers(str: String) -> String {
        str.replace("_pauze", "[PAUSE]")
            .replace("_tyla", "[PAUSE]")
            .replace("_ikvepimas", "[INHALE]")
            .replace("_iskvepimas", "[EXHALE]")
            .replace("_nurijimas", "[SWALLOW]")
            .replace("_cepsejimas", "[SMACK]")
            .replace("_kede", "[CHAIR]")
            .replace("_pilvas", "[STOMACH]")
            .replace("_garsas", "[NOISE]")
            .replace("_puslapis", "[PAGE]")
            .replace("_durys", "[DOOR]")
            .replace("_eh", "[EH]")
            .replace("-", "[MIDWORDPAUSE]")
    }

    fn process_accents(str: String) -> String {
        lazy_static! {
            static ref postfix_accent_re: Regex = Regex::new(r"(?P<last>[^ ])(?P<accent>_[^ ]+)").unwrap();
        }
        postfix_accent_re.replace_all(&str, "$last")
            .replace("_", "")
    }

    fn perform_postprocessing(str: String) -> String {
        lazy_static! {
            static ref multi_space_re: Regex = Regex::new(r" {2,}").unwrap();
        }
        String::from(multi_space_re.replace_all(str.replace("\r", " ")
            .replace("\n", " ")
            .replace("\t", " ")
            .trim(), " "))
    }

    fn convert_to_phonemes(str: &str, resolvers: &Vec<Box<PhonemeResolver>>) -> Vec<Phoneme> {
        let mut result: Vec<Phoneme> = Vec::new();
        let words: Vec<&str> = str.split_whitespace().collect();

        for i in 0..words.len() {
            let word = if words[i].starts_with("[") {
                words[i].to_string()
            } else {
                words[i].to_lowercase()
            };

            for resolver in resolvers.iter() {
                match resolver.resolve(&word) {
                    Some(mut v) => {
                        v.drain(0..)
                            .for_each(|v| result.push(v));
                        if i != words.len()-1 {
                            result.push(Phoneme::from_str(" ").unwrap());
                        }
                        break;
                    },
                    None => continue
                }
            }
        }

        result
    }

    pub fn construct(de: DictEntry, resolvers: &Vec<Box<PhonemeResolver>>) -> Self {
        let mut t = TrainingEntry::fix_encoding_errors(de.transcript);
        t = TrainingEntry::fix_spelling_errors(t);
        t = TrainingEntry::process_markers(t);
        t = TrainingEntry::process_accents(t);
        t = TrainingEntry::perform_postprocessing(t);

        TrainingEntry {
            phonemes: TrainingEntry::convert_to_phonemes(&t, resolvers),
            audio_path: de.audio_path,
            transcript: t
        }
    }
}