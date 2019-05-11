use phonemes::Phoneme;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::Read;
use regex::Regex;
use std::str::FromStr;
use std::error::Error;
use libc::c_void;
use libc::c_char;
use libc::size_t;
use std::ffi::CString;
use model_runner;
use std::ffi::CStr;
use std::collections::HashSet;
use model_def::ModelDef;
use std::iter::FromIterator;

pub trait PhonemeResolver {
    fn resolve(&self, graphemes: &str) -> Option<Vec<Phoneme>>;
}

pub struct DictionaryPhonemeResolver {
    dict: HashMap<String, Vec<Phoneme>>
}

impl DictionaryPhonemeResolver {
    pub fn load(path: &Path) -> io::Result<DictionaryPhonemeResolver> {
        let mut file = File::open(path)?;
        let mut dict_str = String::new();
        file.read_to_string(&mut dict_str)?;

        let mut dict: HashMap<String, Vec<Phoneme>> = HashMap::new();

        lazy_static! {
            static ref postfix_accent_re: Regex = Regex::new(r"^(?P<graph>[^ ]+) +(?P<accent>.+)$").unwrap(); // TODO fix var and named cap group names
        }

        for entry in dict_str.split("\n") {
            let caps = match postfix_accent_re.captures(entry) {
                Some(v) => v,
                None => {
                    warn!("Cannot parse dictionary line \"{}\" as a dictionary entry", entry);
                    continue;
                }
            };

            let gr = caps.get(1).unwrap().as_str();
            let ph: Vec<Phoneme> = caps.get(2).unwrap()
                .as_str().split_whitespace()
                .map(|v| Phoneme::from_str(v).unwrap())
                .collect();

            dict.insert(gr.to_string(), ph);
        }


        Ok(DictionaryPhonemeResolver {
            dict
        })
    }
}

impl PhonemeResolver for DictionaryPhonemeResolver {
    fn resolve(&self, graphemes: &str) -> Option<Vec<Phoneme>> {
        match self.dict.get(graphemes) {
            Some(v) => Some(v.clone().to_vec()),
            None => None
        }
    }
}

pub struct MarkerPhonemeResolver {}

impl MarkerPhonemeResolver {
    pub fn new() -> MarkerPhonemeResolver {
        MarkerPhonemeResolver{}
    }
}

impl PhonemeResolver for MarkerPhonemeResolver {
    fn resolve(&self, graphemes: &str) -> Option<Vec<Phoneme>> {
        match graphemes.starts_with("[") {
            true => {
                let ph = Phoneme::from_str(graphemes).unwrap();
                if ph.valid {
                    Some(vec![ph])
                } else {
                    None
                }
            },
            false => None
        }
    }
}

pub struct DeadEndPhonemeResolver {}

impl DeadEndPhonemeResolver {
    pub fn new() -> DeadEndPhonemeResolver {
        DeadEndPhonemeResolver{}
    }
}

impl PhonemeResolver for DeadEndPhonemeResolver {
    fn resolve(&self, graphemes: &str) -> Option<Vec<Phoneme>> {
        warn!("Failed to resolve phonemes for word \"{}\"", graphemes);
        Some(vec![Phoneme::from_str(graphemes).unwrap()])
    }
}

pub struct DummyPhonemeResolver {}

impl DummyPhonemeResolver {
    pub fn new() -> DummyPhonemeResolver {
        DummyPhonemeResolver{}
    }
}

impl PhonemeResolver for DummyPhonemeResolver {
    fn resolve(&self, _graphemes: &str) -> Option<Vec<Phoneme>> {
        None
    }
}

pub struct TensorflowPhonemeResolver {
    ptr: *const c_void,
    allowed_tokens: HashSet<String>
}

impl TensorflowPhonemeResolver {
    pub fn load(model_folder_path: &Path) -> Result<TensorflowPhonemeResolver, Box<dyn Error>> {
        let model_def: ModelDef = serde_json::from_reader(
            File::open(model_folder_path.join("model.json")).unwrap()
        ).unwrap();
        let path: CString = CString::new(model_folder_path.as_os_str().to_str().unwrap())?;
        Ok(TensorflowPhonemeResolver {
            ptr: unsafe { model_runner::getModelRunnerInstance(path.as_ptr()) },
            allowed_tokens: HashSet::from_iter(model_def.in_tokens)
        })
    }
}

impl Drop for TensorflowPhonemeResolver {
    fn drop(&mut self) {
        unsafe {
            model_runner::deleteModelRunnerInstance(self.ptr)
        }
    }
}

impl PhonemeResolver for TensorflowPhonemeResolver {
    fn resolve(&self, graphemes: &str) -> Option<Vec<Phoneme>> {
        if graphemes.contains("[midwordpause]") {
            let mut result: Vec<Phoneme> = vec![];
            for (idx, part) in graphemes.split("[midwordpause]").enumerate() {
                match self.resolve(part) {
                    Some(mut r) => {
                        if idx != 0 {
                            result.push(Phoneme::from_str("[MIDWORDPAUSE]").unwrap());
                        }
                        r.drain(0..).for_each(|v|result.push(v));
                    },
                    None => return None
                }
            }
            return Some(result);
        }

        let mut phonemes: Vec<String> = Vec::new();
        let mut _graphemes: Vec<CString> = Vec::new();
        let mut last = 0;
        for c in graphemes.chars() {
            let len = c.len_utf8();
            let slice = &graphemes[last..last+len];
            if !self.allowed_tokens.contains(slice) {
                return None;
            }
            _graphemes.push(CString::new(slice).unwrap());
            last += len;
        }
        unsafe {
            let mut grphms: Vec<*const c_char> = vec![std::ptr::null(); graphemes.chars().count()];
            let mut result_size: size_t = 0;
            let mut result: *const *const c_char = std::ptr::null();
            for (idx, c) in _graphemes.iter().enumerate() {
                grphms[idx] = c.as_ptr();
            }
            model_runner::modelRunnerInfer(
                self.ptr,
                grphms.as_ptr(),
                grphms.len(),
                &mut result,
                &mut result_size,
                255
            );

            for i in 0..result_size {
                let c_str: &CStr = CStr::from_ptr(*result.offset(i as isize));
                phonemes.push(c_str.to_str().unwrap().clone().to_owned());
                libc::free(*result.offset(i as isize) as *mut c_void);
            }
            libc::free(result as *mut c_void);
        }

        let result: Vec<Phoneme> = phonemes.iter()
            .skip(1)
            .take(phonemes.len()-2)
            .map(|val| Phoneme::from_str(val).unwrap())
            .collect();

        Some(result)
    }
}
