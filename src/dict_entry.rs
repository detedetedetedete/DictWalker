use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::Read;
use decode::decode_utf16_le;
use decode::decode_windows_1257;
use decode::decode_utf16_be;
use std::collections::HashMap;
use std::fs::read_dir;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
pub struct DictEntry {
    pub name: String,
    pub transcript: String,
    pub containing_dir: String,
    pub audio_path: String,
    pub transcript_path: String
}

impl DictEntry {
    pub fn new_empty() -> DictEntry {
        DictEntry {
            name: String::new(),
            transcript: String::new(),
            containing_dir: String::new(),
            audio_path: String::new(),
            transcript_path: String::new()
        }
    }

    pub fn is_incomplete(&self) -> bool {
        self.name.is_empty() ||
            self.transcript.is_empty() ||
            self.containing_dir.is_empty() ||
            self.audio_path.is_empty() ||
            self.transcript.is_empty()
    }

    fn read_transcript(path: &Path) -> Result<String, String> {
        let mut file = match File::open(path) {
            Ok(v) => v,
            Err(e) => return Err(String::from(e.description()))
        };
        let mut bytes: Vec<u8> = Vec::new();
        match file.read_to_end(&mut bytes) {
            Ok(_) => (),
            Err(e) => return Err(String::from(e.description()))
        };

        match String::from_utf8(bytes.clone()) {
            Ok(v) => Ok(v),
            Err(e) => {
                debug!("Failed to read {:?} as UTF-8 ({}), checking for 0xFF 0xFE bytes...", path, e);
                if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
                    trace!("0xFF 0xFE bytes detected. Trying to decode as UTF-16LE...");
                    match decode_utf16_le(&bytes[2..]) {
                        Ok(v) => Ok(v),
                        Err(e) => {
                            debug!("Failed to read as UTF-16LE: {}", e);
                            trace!("Will try to read as windows 1257...");
                            decode_windows_1257(&bytes)
                        }
                    }
                } else if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
                    trace!("0xFE 0xFF bytes detected. Trying to decode as UTF-16BE...");
                    match decode_utf16_be(&bytes[2..]) {
                        Ok(v) => Ok(v),
                        Err(e) => {
                            debug!("Failed to read as UTF-16BE: {}", e);
                            trace!("Will try to read as windows 1257...");
                            decode_windows_1257(&bytes)
                        }
                    }
                } else {
                    debug!("No 0xFF 0xFE bytes.");
                    trace!("Will try to read as windows 1257...");
                    decode_windows_1257(&bytes)
                }
            }
        }
    }

    // TODO: maybe use the .? syntax to propagate Err up
    pub fn collect_entries(dir: &Path, audio_exts: &HashSet<String>, text_exts: &HashSet<String>) -> Result<Vec<DictEntry>, String> {
        let mut paths: Vec<String> = Vec::new();
        let mut files: Vec<String> = Vec::new();
        let mut entries: HashMap<String, DictEntry> = HashMap::new();

        let dir_str = match dir.to_str() {
            Some(v) => String::from(v),
            None => return Err(format!("Cannot get string representation of path \"{:?}\"", dir))
        };
        if dir.is_dir() {
            paths.push(dir_str);
        } else {
            files.push(dir_str);
        }

        while !paths.is_empty() {
            let p = paths.remove(0);
            debug!("Visiting path \"{}\".", p);

            let dir_entries = match read_dir(p) {
                Ok(v) => v,
                Err(e) => return Err(String::from(e.description()))
            };

            for entry in dir_entries {
                let entry = match entry {
                    Ok(v) => v,
                    Err(e) => return Err(String::from(e.description()))
                };

                let path_str = match entry.path().to_str() {
                    Some(v) => String::from(v),
                    None => return Err(format!("Cannot get string representation of path \"{:?}\"", entry.path()))
                };
                if entry.path().is_dir() {
                    trace!("Adding path \"{}\".", path_str);
                    paths.push(String::from(path_str));
                } else {
                    trace!("Adding file \"{}\".", path_str);
                    files.push(String::from(path_str));
                }
            }
        }

        for file_str in files {
            let file = Path::new(&file_str);

            let file_stem = match file.file_stem() {
                Some(v) => match v.to_str() {
                    Some(v) => v,
                    None => return Err(format!("Cannot get &str from OsStr \"{:?}\"!", v))
                },
                None => return Err(format!("Cannot get file stem from {:?}!", file))
            };

            let mut remove: Option<String> = None;

            {
                let mut entry = entries.entry(String::from(file_stem)).or_insert(DictEntry::new_empty());
                let extension = match file.extension() {
                    Some(v) => match v.to_str() {
                        Some(v) => v,
                        None => return Err(format!("Cannot get &str from OsStr \"{:?}\"!", v))
                    },
                    None => ""
                };

                entry.name = String::from(file_stem);
                entry.containing_dir = match file.parent() {
                    Some(v) => match v.to_str() {
                        Some(v) => String::from(v),
                        None => return Err(format!("Cannot get &str from Path \"{:?}\"!", v))
                    },
                    None => return Err(format!("Cannot resolve containing directory for {:?}!", file))
                };

                if audio_exts.contains(&extension.to_lowercase()) {
                    if !entry.audio_path.is_empty() {
                        return Err(format!("Naming collision: \"{}\" vs \"{}\"!", entry.audio_path, file_str));
                    }
                    entry.audio_path = String::from(file_str.clone());
                } else if text_exts.contains(&extension.to_lowercase()) {
                    if !entry.transcript_path.is_empty() {
                        return Err(format!("Naming collision: \"{}\" vs \"{}\"!", entry.transcript_path, file_str));
                    }
                    entry.transcript_path = String::from(file_str.clone());
                    entry.transcript = DictEntry::read_transcript(file)?;
                } else {
                    warn!("Unknown file extension \"{}\", file {:?}!", extension, file);
                    remove = Some(String::from(file_stem));
                }
            }

            match remove {
                Some(v) => { entries.remove(&v); },
                None => ()
            };
        }

        let values = entries
            .drain()
            .filter(|kv| {
                let incomplete = kv.1.is_incomplete();
                if incomplete {
                    warn!("Incomplete entry: {:#?}", kv.1);
                }
                !incomplete
            })
            .map(|kv| kv.1)
            .collect();
        Ok(values)
    }
}


