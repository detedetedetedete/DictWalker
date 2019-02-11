use phonemes::Phoneme;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::Read;
use regex::Regex;
use std::str::FromStr;
use tensorflow::Graph;
use tensorflow::ImportGraphDefOptions;
use tensorflow::Operation;
use tensorflow::Session;
use tensorflow::SessionOptions;
use tensorflow::SessionRunArgs;
use tensorflow::FetchToken;
use tensorflow::Tensor;
use std::error::Error;
use serde_json;
use model_def::ModelDef;
use io_map::IOMap;

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
    model_def: ModelDef,
    in_map: IOMap,
    out_map: IOMap,
    encoder: Graph,
    encoder_input: Operation,
    encoder_outputs: Vec<Operation>,
    decoder: Graph,
    decoder_inputs: Vec<Operation>,
    decoder_outputs: Vec<Operation>,
    start_token: String,
    end_token: String
}

impl TensorflowPhonemeResolver {
    pub fn load(model_folder_path: &Path) -> Result<TensorflowPhonemeResolver, Box<dyn Error>> {
        let mut model_def: ModelDef = serde_json::from_reader(
            File::open(model_folder_path.join("model.json")).unwrap()
        ).unwrap();
        let mut encoder_proto: Vec<u8> = Vec::new();
        File::open(model_folder_path.join("encoder_inference_model.pb"))?.read_to_end(&mut encoder_proto)?;
        let mut encoder = Graph::new();
        encoder.import_graph_def(&encoder_proto, &ImportGraphDefOptions::new())?;
        let encoder_input = encoder.operation_by_name_required("encoder_input")?;
        let mut encoder_outputs: Vec<Operation> = Vec::new();
        for op in encoder.operation_iter() {
            if op.name().unwrap().ends_with("_output") {
                encoder_outputs.push(op);
            }
        }

        let mut decoder_proto: Vec<u8> = Vec::new();
        File::open(model_folder_path.join("decoder_inference_model.pb"))?.read_to_end(&mut decoder_proto)?;
        let mut decoder = Graph::new();
        decoder.import_graph_def(&decoder_proto, &ImportGraphDefOptions::new())?;
        let mut decoder_inputs: Vec<Operation> = Vec::new();
        let mut decoder_outputs: Vec<Operation> = Vec::new();
        decoder_inputs.push(decoder.operation_by_name_required("decoder_input")?);
        for op in decoder.operation_iter() {
            let op_name = op.name().unwrap();
            if op_name.ends_with("_state_h") || op_name.ends_with("_state_c") {
                decoder_inputs.push(op);
            } else if op_name.ends_with("_output") {
                decoder_outputs.push(op);
            }
        }

        let start_token = "[S]".to_string();
        let end_token = "[E]".to_string();
        model_def.out_tokens.push(start_token.clone());
        model_def.out_tokens.push(end_token.clone());

        Ok(TensorflowPhonemeResolver {
            in_map: IOMap::new(model_def.in_tokens.clone()),
            out_map: IOMap::new(model_def.out_tokens.clone()),
            model_def,
            encoder,
            encoder_input,
            encoder_outputs,
            decoder,
            decoder_inputs,
            decoder_outputs,
            start_token,
            end_token
        })
    }
}

impl PhonemeResolver for TensorflowPhonemeResolver {
    fn resolve(&self, graphemes: &str) -> Option<Vec<Phoneme>> {
        let mut encoder_session = Session::new(&SessionOptions::new(), &self.encoder).unwrap();

        let mut encoder_input: Tensor<f32> = Tensor::new(&[1, self.model_def.max_in_length as u64, self.in_map.len() as u64]);

        let g: Vec<String> = graphemes.chars().map(|a| a.to_string()).collect();
        self.in_map.encode(
            g.iter(),
            self.model_def.max_in_length,
            &mut encoder_input
        );

        let mut encoder_args = SessionRunArgs::new();
        encoder_args.add_feed(
            &self.encoder_input,
            0,
        &encoder_input
        );

        let mut encoder_fetch_tokens: Vec<FetchToken> = Vec::new();
        for encoder_output in &self.encoder_outputs {
            encoder_fetch_tokens.push(
                encoder_args.request_fetch(&encoder_output, 0)
            );
        }

        encoder_session.run(&mut encoder_args).unwrap();
        let mut states_vec: Vec<Tensor<f32>> = Vec::new();
        for encoder_fetch_token in encoder_fetch_tokens {
            states_vec.push(encoder_args.fetch(encoder_fetch_token).unwrap());
        }

        let mut last_output: Tensor<f32> = Tensor::new(&[1, self.model_def.max_out_length as u64, self.out_map.len() as u64]);
        let mut decoder_session = Session::new(&SessionOptions::new(), &self.decoder).unwrap();
        let mut decoder_args = SessionRunArgs::new();
        self.out_map.encode(
            vec![self.start_token.clone()].iter(),
            self.model_def.max_out_length,
            &mut last_output
        );


        while self.out_map.decode(&last_output)[0] != self.end_token {
            let mut decoder_fetch_tokens: Vec<FetchToken> = Vec::new();
            for decoder_output in &self.decoder_outputs {
                decoder_fetch_tokens.push(
                    decoder_args.request_fetch(&decoder_output, 0)
                );
            }
            for (idx, dec_input) in self.decoder_inputs.iter().enumerate() {
                decoder_args.add_feed(
                    dec_input,
                    0,
                    match idx {
                        0 => &last_output,
                        _ => &states_vec[idx-1]
                    }
                );
            }
            decoder_session.run(&mut decoder_args).unwrap();
            for (idx, decoder_fetch_token) in decoder_fetch_tokens.iter().enumerate() {
                if idx == 0 {
                    last_output = decoder_args.fetch(*decoder_fetch_token).unwrap();
                } else {
                    states_vec.push(decoder_args.fetch(*decoder_fetch_token).unwrap());
                }
            }
        }

        None

    }
}