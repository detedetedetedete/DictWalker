use std::collections::HashMap;
use tensorflow::Tensor;
use std::slice::Iter;


pub struct IOMap {
    map: HashMap<String, usize>,
    keys: Vec<String>
}

impl IOMap {
    pub fn new(keys: Vec<String>) -> IOMap {
        let mut map: HashMap<String, usize> = HashMap::new();

        for (idx, key) in keys.iter().enumerate() {
            map.insert(key.clone(), idx);
        }

        IOMap {
            map,
            keys
        }
    }

    pub fn encode(&self, string: Iter<String>, length: usize, tensor: &mut Tensor<f32>) {
        let mlen = self.map.len();
        let mut chunks = tensor.chunks_mut(mlen);

        for (idx, ch) in string.enumerate() {
            let mut chunk = chunks.nth(idx).unwrap();
            chunk[self.map[ch]] = 1.;
        }
    }

    pub fn decode(&self, tensor: &Tensor<f32>) -> Vec<String> {
        let chunks = tensor.chunks(self.map.len());
        let mut result = Vec::new();

        for chunk in chunks {
            let argmax = chunk
                .iter()
                .enumerate()
                .max_by(|&(_, a), &(_, b)| a.partial_cmp(b).unwrap())
                .unwrap().0;
            result.push(self.keys[argmax].clone());
        }

        result
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }
}