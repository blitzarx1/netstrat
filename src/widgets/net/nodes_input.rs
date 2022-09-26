#[derive(PartialEq, Clone, Default)]
pub struct NodesInput {
    pub input: String,
}

impl NodesInput {
    pub fn splitted(&self) -> Vec<String> {
        if self.input.is_empty() {
            return vec![];
        }
        self.input
            .split(',')
            .map(|el| el.trim().to_string())
            .collect()
    }
}
