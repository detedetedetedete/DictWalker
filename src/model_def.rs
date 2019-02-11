#[derive(Serialize, Deserialize)]
pub struct ModelDef {
    pub name: String,
    pub in_tokens: Vec<String>,
    pub out_tokens: Vec<String>,
    pub max_in_length: usize,
    pub max_out_length: usize
}